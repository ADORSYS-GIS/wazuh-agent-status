//! Agent state manager — owns the single source of truth for local agent state,
//! broadcasts changes to subscribers, and provides on-demand version checking.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{RwLock, broadcast};
use tokio::time;
use tracing::{info, warn};

use crate::config::{AgentPaths, Config};
use crate::models::{AgentState, ComponentUpdate, LogLine, UpdateStatus, VersionInfo};
use crate::status_provider::StatusProvider;
use crate::version_utils::fetch_version_info;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

// ── Version cache ─────────────────────────────────────────────────────────────

struct VersionCache {
    /// The structured update status sent to clients.
    status: UpdateStatus,
    /// The raw manifest data (stored for re-computation if local version changes).
    info: VersionInfo,
    /// When this cache entry was populated.
    fetched_at: Instant,
}

// ── AgentManager ──────────────────────────────────────────────────────────────

/// Central manager: owns local state, broadcasts changes, serves version info.
pub struct AgentManager {
    /// The most recent local agent state snapshot.
    state: Arc<RwLock<AgentState>>,
    /// Notifies all `subscribe-status` subscribers on state change.
    notifier: broadcast::Sender<AgentState>,
    /// Platform-specific local status reader.
    provider: Box<dyn StatusProvider>,
    /// Cached result of the last remote version check.
    version_cache: RwLock<Option<VersionCache>>,
    /// Platform-specific file paths.
    paths: Arc<AgentPaths>,
    /// Runtime configuration.
    config: Arc<Config>,
    local_agent_id: String,
    local_agent_name: String,
    local_agent_key: String,
}

impl AgentManager {
    /// Create a new manager using the native status provider for the current OS.
    #[must_use]
    pub fn new(config: Arc<Config>, paths: Arc<AgentPaths>) -> Self {
        let provider = Box::new(crate::status_provider::native_provider(
            paths.as_ref().clone(),
        ));
        Self::new_custom(config, paths, provider)
    }

    /// Create a new manager with a custom status provider.
    ///
    /// This is a professional extension point that also facilitates integration
    /// testing without polluting the production logic with test hooks.
    #[must_use]
    pub fn new_custom(
        config: Arc<Config>,
        paths: Arc<AgentPaths>,
        provider: Box<dyn StatusProvider>,
    ) -> Self {
        let (tx, _) = broadcast::channel(128);

        let (agent_id, agent_name, agent_key) = Self::read_client_keys(&paths);
        let initial_state = AgentState {
            agent_id: agent_id.clone(),
            agent_name: agent_name.clone(),
            agent_key: agent_key.clone(),
            ..Default::default()
        };

        Self {
            state: Arc::new(RwLock::new(initial_state)),
            notifier: tx,
            provider,
            version_cache: RwLock::new(None),
            paths,
            config,
            local_agent_id: agent_id,
            local_agent_name: agent_name,
            local_agent_key: agent_key,
        }
    }

    fn read_client_keys(paths: &AgentPaths) -> (String, String, String) {
        match std::fs::read_to_string(&paths.client_keys) {
            Ok(content) => {
                if let Some(first_line) = content.lines().next() {
                    let parts: Vec<&str> = first_line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        return (
                            parts[0].to_string(),
                            parts[1].to_string(),
                            parts[3].to_string(),
                        );
                    }
                    if parts.len() >= 2 {
                        return (parts[0].to_string(), parts[1].to_string(), String::new());
                    }
                }
                warn!("client.keys file is empty or malformed");
                (String::new(), String::new(), String::new())
            }
            Err(e) => {
                warn!(path = %paths.client_keys.display(), error = %e, "Could not read client.keys");
                (String::new(), String::new(), String::new())
            }
        }
    }

    // ── State access ──────────────────────────────────────────────────────────

    /// Return the runtime configuration.
    pub fn config(&self) -> Arc<Config> {
        Arc::clone(&self.config)
    }

    /// Return a snapshot of the current local agent state.
    pub async fn get_state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    /// Subscribe to state-change notifications.
    ///
    /// Each subscriber gets their own [`broadcast::Receiver`]. The channel
    /// has a capacity of 128 updates; slow clients will receive a
    /// [`broadcast::error::RecvError::Lagged`] if they fall behind.
    pub fn subscribe(&self) -> broadcast::Receiver<AgentState> {
        self.notifier.subscribe()
    }

    // ── Polling ───────────────────────────────────────────────────────────────

    /// Continuously poll the local agent state at the configured interval.
    ///
    /// This loop performs **only local** operations (file reads / process
    /// checks) — no network I/O.  Online version checking is done on-demand
    /// via [`get_version_status`].
    pub async fn start_polling(&self) {
        let mut ticker = time::interval(self.config.poll_interval);
        let mut last_healing_attempt: Option<Instant> = None;

        loop {
            ticker.tick().await;
            match self.provider.get_partial_state() {
                Ok(new_state) => {
                    let mut current = self.state.write().await;

                    // Self-healing: if agent is stopped, try to restart it (if enabled)
                    if self.config.self_healing
                        && new_state.status == crate::models::AgentStatus::Inactive
                    {
                        let now = Instant::now();
                        let should_attempt = match last_healing_attempt {
                            Some(last) => now.duration_since(last) > time::Duration::from_secs(300), // 5-minute cooldown
                            None => true,
                        };

                        if should_attempt {
                            info!("Self-healing: Wazuh agent is inactive. Attempting restart...");
                            last_healing_attempt = Some(now);

                            let (cmd_name, args): (&str, Vec<String>) =
                                if cfg!(target_os = "windows") {
                                    (
                                        "powershell.exe",
                                        vec![
                                            "-NoProfile".into(),
                                            "-NonInteractive".into(),
                                            "-Command".into(),
                                            "Restart-Service -Name WazuhSvc -Force".into(),
                                        ],
                                    )
                                } else {
                                    (
                                        "sudo",
                                        vec![
                                            self.paths.wazuh_control.to_string_lossy().into_owned(),
                                            "restart".into(),
                                        ],
                                    )
                                };
                            tokio::spawn(async move {
                                let mut cmd = Command::new(cmd_name);
                                cmd.args(&args);

                                match cmd.output().await {
                                    Ok(o) => {
                                        if o.status.success() {
                                            info!(
                                                "Self-healing: Restart command executed successfully"
                                            );
                                        } else {
                                            warn!(
                                                "Self-healing: Restart command failed with exit code {}: {}",
                                                o.status.code().unwrap_or(-1),
                                                String::from_utf8_lossy(&o.stderr)
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Self-healing: Failed to spawn restart command: {e}")
                                    }
                                }
                            });
                        }
                    } else if new_state.status == crate::models::AgentStatus::Active {
                        // Just broadcast state; don't reset healing clock to maintain strict cooldown
                    }

                    let mut final_state = new_state;
                    final_state.self_healing_enabled = self.config.self_healing;
                    final_state.agent_id = self.local_agent_id.clone();
                    final_state.agent_name = self.local_agent_name.clone();
                    final_state.agent_key = self.local_agent_key.clone();

                    if *current != final_state {
                        info!(state = ?final_state, "Agent state changed");
                        *current = final_state.clone();
                        let _ = self.notifier.send(final_state);
                    }
                }
                Err(e) => warn!("Failed to poll agent status: {e}"),
            }
        }
    }

    // ── Update Execution ──────────────────────────────────────────────────────

    /// Initiate an update process and return a stream of log output.
    pub async fn initiate_update(&self, is_prerelease: bool) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel(100);
        let paths = Arc::clone(&self.paths);

        // If prerelease, fetch the version string before spawning the task to avoid lifetime issues
        let prerelease_version = if is_prerelease {
            let status = self.get_version_status().await;
            Some(status.tray.latest_version)
        } else {
            None
        };

        info!(is_prerelease, update_script = %paths.update_script.display(), "Spawning update task");

        tokio::spawn(async move {
            info!("Update task started, sending initial progress message");
            if let Err(e) = tx
                .send("UPDATE_PROGRESS: [STATUS] Starting update process...".to_string())
                .await
            {
                warn!(error = %e, "Failed to send initial progress message");
                return;
            }
            info!("Initial progress message sent successfully");

            // Determine the script path first, then build the command
            let script_path: std::path::PathBuf;
            // Track the tag for prerelease updates so the setup script downloads
            // components and version.txt from the correct release tag
            let prerelease_tag: Option<String>;

            if is_prerelease {
                let version = match prerelease_version {
                    Some(v) if v != "Unknown" => v,
                    _ => {
                        warn!("Could not determine latest prerelease version");
                        let _ = tx.send("UPDATE_PROGRESS: [FAILURE] Could not determine latest prerelease version".to_string()).await;
                        return;
                    }
                };

                info!(version = %version, "Processing prerelease update");
                prerelease_tag = Some(format!("refs/tags/v{version}"));

                let _ = tx
                    .send(format!(
                        "UPDATE_PROGRESS: [STATUS] Downloading setup script for v{}...",
                        version
                    ))
                    .await;
                let url = if cfg!(target_os = "windows") {
                    format!(
                        "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v{}/scripts/windows/setup-agent.ps1",
                        version
                    )
                } else {
                    format!(
                        "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v{}/scripts/setup-agent.sh",
                        version
                    )
                };

                match crate::http::fetch_bytes(&url, Duration::from_secs(30)).await {
                    Ok(bytes) => {
                        let tmp_script: std::path::PathBuf = if cfg!(target_os = "windows") {
                            let mut tmp_dir = std::env::temp_dir();
                            tmp_dir.push(format!("setup-agent-{}.ps1", version));
                            tmp_dir
                        } else {
                            let mut tmp_dir = std::env::temp_dir();
                            tmp_dir.push(format!("setup-agent-{}.sh", version));
                            tmp_dir
                        };

                        info!(script_path = %tmp_script.display(), "Saving setup script to temporary file");

                        let save_result = if cfg!(target_os = "windows") {
                            tokio::fs::write(&tmp_script, bytes).await
                        } else {
                            let cmd_str = format!("cat > {}", tmp_script.display());
                            let mut cmd = Command::new("sudo");
                            cmd.arg("sh")
                                .arg("-c")
                                .arg(&cmd_str)
                                .stdin(Stdio::piped())
                                .stdout(Stdio::null())
                                .stderr(Stdio::piped());
                            match cmd.spawn() {
                                Ok(mut child) => {
                                    if let Some(mut stdin) = child.stdin.take() {
                                        let _ = stdin.write_all(&bytes).await;
                                        drop(stdin);
                                    }
                                    match child.wait_with_output().await {
                                        Ok(out) if out.status.success() => Ok(()),
                                        Ok(out) => Err(std::io::Error::other(
                                            String::from_utf8_lossy(&out.stderr).into_owned(),
                                        )),
                                        Err(e) => Err(e),
                                    }
                                }
                                Err(e) => Err(e),
                            }
                        };

                        if let Err(e) = save_result {
                            warn!(error = %e, "Failed to save setup script");
                            let _ = tx
                                .send(format!(
                                    "UPDATE_PROGRESS: [FAILURE] Failed to save setup script: {e}"
                                ))
                                .await;
                            return;
                        }
                        // Make executable on Unix
                        if !cfg!(target_os = "windows") {
                            let _ = std::process::Command::new("chmod")
                                .arg("+x")
                                .arg(&tmp_script)
                                .status();
                        }

                        info!(script = %tmp_script.display(), "Executing prerelease setup script");
                        let _ = tx
                            .send(
                                "UPDATE_PROGRESS: [STATUS] Executing prerelease setup..."
                                    .to_string(),
                            )
                            .await;
                        script_path = std::path::PathBuf::from(&tmp_script);
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to download setup script");
                        let _ = tx
                            .send(format!(
                                "UPDATE_PROGRESS: [FAILURE] Failed to download setup script: {e}"
                            ))
                            .await;
                        return;
                    }
                }
            } else {
                info!(script = %paths.update_script.display(), "Executing standard update script");
                prerelease_tag = None;
                if cfg!(target_os = "windows") {
                    let tmp_script = std::env::temp_dir().join("adorsys-update.ps1");

                    let _ = tx
                        .send("UPDATE_PROGRESS: [STATUS] Downloading fresh Windows update wrapper...".to_string())
                        .await;

                    if let Err(e) = tokio::fs::write(
                        &tmp_script,
                        include_str!("../../scripts/windows/adorsys-update.ps1"),
                    )
                    .await
                    {
                        let _ = tx
                            .send(format!(
                                "UPDATE_PROGRESS: [FAILURE] Failed to save update wrapper: {e}"
                            ))
                            .await;
                        return;
                    }
                    script_path = tmp_script;
                } else {
                    script_path = paths.update_script.clone();
                }
            }

            // Build the command — platform-specific execution
            let mut cmd = if cfg!(target_os = "windows") {
                let script_str = script_path.to_str().unwrap_or_default();
                if script_str.ends_with(".ps1") {
                    info!("Running PowerShell script directly");
                    let mut c = Command::new("powershell.exe");
                    c.args([
                        "-NoProfile",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                        script_str,
                    ]);
                    // Always pass -Update so the setup script runs in upgrade mode
                    c.arg("-Update");
                    // Set the tag so the setup script downloads components from the correct release
                    if let Some(ref tag) = prerelease_tag {
                        c.env("WAZUH_AGENT_REPO_REF", tag);
                    }
                    c
                } else {
                    info!("Running batch script directly");
                    let mut c = Command::new("cmd.exe");
                    let mut cmd_line = format!("\"{}\" -Update", script_path.display());
                    if prerelease_tag.is_some() {
                        cmd_line.push_str(" -Prerelease");
                    }
                    c.arg("/C").arg(cmd_line);
                    if let Some(ref tag) = prerelease_tag {
                        c.env("WAZUH_AGENT_REPO_REF", tag);
                    }
                    c
                }
            } else {
                let is_root = std::process::Command::new("id")
                    .arg("-u")
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
                    .unwrap_or(false);

                if is_root {
                    info!("Running as root — executing update script directly");
                    let mut c = Command::new(script_path.as_os_str());
                    // Direct execution: set env var directly on child process
                    if let Some(ref tag) = prerelease_tag {
                        c.env("WAZUH_AGENT_REPO_REF", tag);
                    }
                    c
                } else {
                    info!("Running as non-root — using sudo for update script");
                    let mut c = Command::new("sudo");
                    // sudo resets the environment by default (env_reset), so .env() won't work.
                    // Use sudo's native VAR=value command syntax which is universally supported.
                    if let Some(ref tag) = prerelease_tag {
                        c.arg(format!("WAZUH_AGENT_REPO_REF={}", tag));
                    }
                    c.arg(script_path.as_os_str());
                    c
                }
            };
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

            info!("Spawning update command");
            match cmd.spawn() {
                Ok(mut child) => {
                    info!("Update command spawned successfully");
                    let stdout = child.stdout.take().unwrap();
                    let stderr = child.stderr.take().unwrap();
                    let tx_clone = tx.clone();

                    let windows_response_log = if cfg!(target_os = "windows") {
                        std::path::PathBuf::from(
                            r"C:\Program Files (x86)\ossec-agent\active-response\active-responses.log",
                        )
                    } else {
                        std::path::PathBuf::new()
                    };

                    async fn append_update_log(path: &std::path::Path, line: &str) {
                        if path.as_os_str().is_empty() {
                            return;
                        }

                        if let Some(parent) = path.parent() {
                            let _ = tokio::fs::create_dir_all(parent).await;
                        }

                        if let Ok(mut file) = tokio::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(path)
                            .await
                        {
                            let _ = file.write_all(format!("{}\n", line).as_bytes()).await;
                        }
                    }

                    // Pipe stdout
                    let windows_response_log_stdout = windows_response_log.clone();
                    tokio::spawn(async move {
                        let mut reader = BufReader::new(stdout).lines();
                        while let Ok(Some(line)) = reader.next_line().await {
                            info!(line = %line, "Update stdout");
                            append_update_log(&windows_response_log_stdout, &line).await;
                            let _ = tx_clone.send(format!("UPDATE_PROGRESS: {}", line)).await;
                        }
                    });

                    // Pipe stderr
                    let tx_clone = tx.clone();
                    let windows_response_log_stderr = windows_response_log.clone();
                    tokio::spawn(async move {
                        let mut reader = BufReader::new(stderr).lines();
                        while let Ok(Some(line)) = reader.next_line().await {
                            warn!(line = %line, "Update stderr");
                            append_update_log(&windows_response_log_stderr, &line).await;
                            let _ = tx_clone
                                .send(format!("UPDATE_PROGRESS: [ERROR] {}", line))
                                .await;
                        }
                    });

                    // Tail the active-responses.log since adorsys-update.sh writes there instead of stdout
                    let active_response_log = if cfg!(target_os = "macos") {
                        std::path::PathBuf::from("/Library/Ossec/logs/active-responses.log")
                    } else if cfg!(target_os = "linux") {
                        std::path::PathBuf::from("/var/ossec/logs/active-responses.log")
                    } else {
                        std::path::PathBuf::new() // Windows doesn't need this, outputs to stdout
                    };

                    let (kill_tx, mut kill_rx) = tokio::sync::oneshot::channel::<()>();
                    if !active_response_log.as_os_str().is_empty() {
                        let tx_log = tx.clone();
                        tokio::spawn(async move {
                            let initial_len = match tokio::fs::metadata(&active_response_log).await
                            {
                                Ok(m) => m.len(),
                                Err(_) => 0,
                            };

                            let mut file = match tokio::fs::File::open(&active_response_log).await {
                                Ok(f) => f,
                                Err(_) => return,
                            };

                            let _ = file.seek(std::io::SeekFrom::Start(initial_len)).await;
                            let mut reader = BufReader::new(file);
                            let mut line = String::new();

                            loop {
                                tokio::select! {
                                    _ = &mut kill_rx => break,
                                    res = reader.read_line(&mut line) => {
                                        match res {
                                            Ok(0) => {
                                                tokio::time::sleep(Duration::from_millis(200)).await;
                                            }
                                            Ok(_) => {
                                                let t = line.trim();
                                                if !t.is_empty() {
                                                    let _ = tx_log.send(format!("UPDATE_PROGRESS: {}", t)).await;
                                                }
                                                line.clear();
                                            }
                                            Err(_) => break,
                                        }
                                    }
                                }
                            }
                        });
                    }

                    match child.wait().await {
                        Ok(status) if status.success() => {
                            let _ = kill_tx.send(());
                            info!(exit_code = ?status.code(), "Update script completed successfully");
                            append_update_log(&windows_response_log, "[SUCCESS] Update completed successfully")
                            .await;
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            let _ = tx
                                .send(
                                    "UPDATE_PROGRESS: [SUCCESS] Update completed successfully"
                                        .to_string(),
                                )
                                .await;
                        }
                        Ok(status) => {
                            let _ = kill_tx.send(());
                            warn!(exit_code = ?status.code(), "Update script failed");
                            append_update_log(
                                &windows_response_log,
                                &format!("[FAILURE] Update script exited with code: {:?}", status.code()),
                            )
                            .await;
                            let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Update script exited with code: {:?}", status.code())).await;
                        }
                        Err(e) => {
                            let _ = kill_tx.send(());
                            warn!(error = %e, "Failed to wait for update script");
                            append_update_log(&windows_response_log, &format!("[FAILURE] Failed to wait for update script: {e}"))
                            .await;
                            let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Failed to wait for update script: {e}")).await;
                        }
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Failed to spawn update command");
                    let error_hint = if cfg!(target_os = "windows") {
                        "check that PowerShell and the script path are available"
                    } else {
                        "check sudoers configuration"
                    };
                    let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Failed to start update script ({error_hint}): {e}")).await;
                }
            }
        });

        rx
    }

    // ── Log streaming ─────────────────────────────────────────────────────────

    /// Open `ossec.log`, seek to the end, and stream new lines as they are
    /// appended.  Returns an [`mpsc::Receiver`] that yields structured
    /// [`LogLine`] values until the file is closed or the client disconnects.
    pub async fn stream_logs(&self) -> mpsc::Receiver<LogLine> {
        let (tx, rx) = mpsc::channel(256);
        let log_path = self.paths.ossec_log.clone();

        tokio::spawn(async move {
            const HISTORY: usize = 50;

            // Verify file exists before attempting anything.
            if !tokio::fs::try_exists(&log_path).await.unwrap_or(false) {
                let _ = tx
                    .send(LogLine::from_raw(format!(
                        "[ERROR] Log file not found: {}",
                        log_path.display()
                    )))
                    .await;
                return;
            }

            // Send last N historical lines so the UI isn't empty on first connect.
            match tokio::fs::read_to_string(&log_path).await {
                Ok(content) => {
                    let mut hist: Vec<&str> = Vec::with_capacity(HISTORY);
                    for line in content.lines() {
                        hist.push(line);
                        if hist.len() > HISTORY {
                            hist.remove(0);
                        }
                    }
                    for line in hist {
                        if tx.send(LogLine::from_raw(line.to_string())).await.is_err() {
                            return;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(LogLine::from_raw(format!(
                            "[WARNING] Could not read history (file may be locked): {e}"
                        )))
                        .await;
                }
            }

            // Re-open and tail from EOF for live lines.
            let file = match tokio::fs::File::open(&log_path).await {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx
                        .send(LogLine::from_raw(format!(
                            "[ERROR] Cannot open log file for tailing: {e}"
                        )))
                        .await;
                    return;
                }
            };
            let mut reader = BufReader::new(file);
            if let Err(e) = reader.seek(std::io::SeekFrom::End(0)).await {
                let _ = tx
                    .send(LogLine::from_raw(format!(
                        "[ERROR] Cannot seek log file: {e}"
                    )))
                    .await;
                return;
            }

            let mut lines = reader.lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        if tx.send(LogLine::from_raw(line)).await.is_err() {
                            break; // Client disconnected
                        }
                    }
                    Ok(None) => {
                        // EOF — wait briefly for new data to be appended.
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(LogLine::from_raw(format!(
                                "[ERROR] Failed to read log line: {e}"
                            )))
                            .await;
                        break;
                    }
                }
            }
        });

        rx
    }

    // ── On-demand version check ───────────────────────────────────────────────

    /// Return the human-readable version status string.
    ///
    /// Results are cached for `config.version_cache_ttl` to avoid hammering
    /// the remote manifest endpoint. The cache is invalidated if the local
    /// version changes (e.g., after an update or manual version file modification).
    pub async fn get_version_status(&self) -> UpdateStatus {
        let now = Instant::now();
        let current_state = self.get_state().await;

        // 1. Try to return fresh cached value (but invalidate if local version changed)
        {
            let cache = self.version_cache.read().await;
            if let Some(c) = &*cache {
                let cache_is_fresh =
                    now.duration_since(c.fetched_at) < self.config.version_cache_ttl;
                // Invalidate cache if local version has changed since cache was created
                let local_version_changed =
                    c.status.tray.current_version != current_state.tray_version;

                if cache_is_fresh && !local_version_changed {
                    return c.status.clone();
                }

                if local_version_changed {
                    info!(
                        old_cached = ?c.status.tray.current_version,
                        current = ?current_state.tray_version,
                        "Local version changed since cache was created; invalidating cache"
                    );
                }
            }
        }

        // 2. Fetch fresh data
        info!(
            "Fetching fresh version manifest from {}",
            self.config.version_url
        );
        let (new_info, is_fallback) = match fetch_version_info(&self.config.version_url).await {
            Some(info) => (Some(info), false),
            None => {
                // Fallback: use last known good info if available
                let cache = self.version_cache.read().await;
                (cache.as_ref().map(|c| c.info.clone()), true)
            }
        };

        match new_info {
            Some(info) => {
                let show_prerelease =
                    crate::version_utils::should_show_prerelease(&info, &current_state.groups);

                let check_update = |name: &str, local_version: &str| {
                    if local_version == "Unknown" || local_version == "Not Installed" {
                        return crate::models::ComponentUpdate {
                            name: name.to_string(),
                            current_version: local_version.to_string(),
                            latest_version: info.framework.version.to_string(),
                            state: crate::models::UpdateState::Unknown,
                            can_update: false,
                        };
                    }

                    let is_outdated = !info.framework.version.is_empty()
                        && info.framework.version != "Unknown"
                        && crate::version_utils::is_version_higher(
                            &info.framework.version,
                            local_version,
                        );

                    let has_prerelease = !info.framework.prerelease_version.is_empty()
                        && show_prerelease
                        && crate::version_utils::is_version_higher(
                            &info.framework.prerelease_version,
                            local_version,
                        );

                    let (state, latest, can_update) = if is_outdated {
                        (
                            crate::models::UpdateState::Outdated,
                            info.framework.version.to_string(),
                            true,
                        )
                    } else if has_prerelease {
                        (
                            crate::models::UpdateState::PrereleaseAvailable,
                            info.framework.prerelease_version.to_string(),
                            true,
                        )
                    } else {
                        (
                            crate::models::UpdateState::UpToDate,
                            info.framework.version.to_string(),
                            false,
                        )
                    };

                    crate::models::ComponentUpdate {
                        name: name.to_string(),
                        current_version: local_version.to_string(),
                        latest_version: latest,
                        state,
                        can_update,
                    }
                };

                let tray_update = check_update("Wazuh Setup", &current_state.tray_version);

                let has_updates = tray_update.can_update;
                let status = UpdateStatus {
                    tray: tray_update,
                    has_updates,
                };

                let mut cache = self.version_cache.write().await;
                *cache = Some(VersionCache {
                    status: status.clone(),
                    info,
                    fetched_at: if is_fallback {
                        self.version_cache
                            .read()
                            .await
                            .as_ref()
                            .map(|c| c.fetched_at)
                            .unwrap_or(now)
                    } else {
                        now
                    },
                });
                status
            }
            None => {
                warn!("Failed to fetch remote version manifest and no cache available");
                UpdateStatus {
                    tray: ComponentUpdate {
                        name: "Wazuh Agent Status".to_string(),
                        current_version: current_state.tray_version,
                        latest_version: "Unknown".to_string(),
                        state: crate::models::UpdateState::Unknown,
                        can_update: false,
                    },
                    has_updates: false,
                }
            }
        }
    }
}
