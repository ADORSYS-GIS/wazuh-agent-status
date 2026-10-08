//! Agent state manager — owns the single source of truth for local agent state,
//! broadcasts changes to subscribers, and provides on-demand version checking.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tokio::sync::{RwLock, broadcast};
use tokio::time;
use tracing::{debug, info, warn};

use crate::config::{AgentPaths, Config};
use crate::models::{AgentState, ComponentUpdate, LogLine, UpdateStatus, VersionInfo};
use crate::status_provider::StatusProvider;
use crate::version_utils::{fetch_plain_version, fetch_version_info};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

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

/// Read the configured Wazuh manager address from ossec.conf so update scripts
/// reinstall the agent against the same manager instead of the wazuh.example.com
/// default. Matches the simple `<address>` extraction the install scripts use.
fn read_configured_manager() -> Option<String> {
    let path = if cfg!(target_os = "windows") {
        r"C:\Program Files (x86)\ossec-agent\ossec.conf"
    } else if cfg!(target_os = "macos") {
        "/Library/Ossec/etc/ossec.conf"
    } else {
        "/var/ossec/etc/ossec.conf"
    };

    let content = std::fs::read_to_string(path).ok()?;
    let start = content.find("<address>")? + "<address>".len();
    let end = content[start..].find("</address>")?;
    let manager = content[start..start + end].trim();
    if manager.is_empty() {
        None
    } else {
        Some(manager.to_string())
    }
}

/// Hard cap on how long an update script may run before it is considered stuck
/// and terminated. The scripts also self-guard with internal timeouts; this is
/// the last line of defence so a hung script can never wedge the UI forever.
const UPDATE_TIMEOUT: Duration = Duration::from_secs(600);

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
    /// Guards against triggering a second auto-update while one is already running.
    pub update_in_progress: Arc<AtomicBool>,
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
            update_in_progress: Arc::new(AtomicBool::new(false)),
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
    /// via [`get_version_status`] and periodically via the auto-update ticker.
    pub async fn start_polling(&self) {
        let mut ticker = time::interval(self.config.poll_interval);
        let mut last_healing_attempt: Option<Instant> = None;

        // Separate ticker for the periodic auto-update check (default: every 30 min)
        let mut update_ticker = time::interval(self.config.auto_update_check_interval);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
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
                _ = update_ticker.tick() => {
                    info!("Periodic auto-update check triggered");
                    self.check_and_auto_update().await;
                }
            }
        }
    }

    /// Check the remote version and automatically run the update script if a
    /// newer version is available.
    ///
    /// - **Stable**: fetches the plain `version.txt` and compares against local.
    /// - **Pre-release**: fetches `versions.json` and checks the agent's group
    ///   membership to decide whether a pre-release update applies.
    ///
    /// A second auto-update will not be triggered while one is already running
    /// (guarded by `update_in_progress`).
    pub async fn check_and_auto_update(&self) {
        // Guard: skip if an update is already running
        if self
            .update_in_progress
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            info!("Auto-update check skipped: an update is already in progress");
            return;
        }

        // RAII guard that clears the flag when dropped (normal return or early return)
        struct UpdateGuard(Arc<AtomicBool>);
        impl Drop for UpdateGuard {
            fn drop(&mut self) {
                self.0.store(false, Ordering::SeqCst);
            }
        }
        let _guard = UpdateGuard(Arc::clone(&self.update_in_progress));

        let status = self.get_version_status().await;

        if !status.has_updates {
            info!(
                "Auto-update check: already up-to-date (local: {}, latest: {})",
                status.tray.current_version, status.tray.latest_version
            );
            return;
        }

        let is_prerelease = matches!(
            status.tray.state,
            crate::models::UpdateState::PrereleaseAvailable
        );

        if cfg!(target_os = "windows") {
            info!("Auto-update: newer version detected. Waiting indefinitely for GUI trigger.");
            return;
        }

        info!(
            local = %status.tray.current_version,
            latest = %status.tray.latest_version,
            is_prerelease,
            "Auto-update: newer version detected, triggering update script"
        );

        let mut rx = self.initiate_update(is_prerelease, false).await;
        // Drain the log channel so the update runs to completion
        while let Some(line) = rx.recv().await {
            info!(target: "auto_update", "{}", line);
        }
    }

    // ── Update Execution ──────────────────────────────────────────────────────

    /// Initiate an update process and return a stream of log output.

    fn get_prerelease_url(version: &str) -> String {
        if cfg!(target_os = "windows") {
            format!(
                "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v{}/scripts/windows/setup-agent.ps1",
                version
            )
        } else {
            format!(
                "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v{}/scripts/setup-agent.sh",
                version
            )
        }
    }

    fn get_prerelease_tmp_script(version: &str) -> std::path::PathBuf {
        let mut tmp_dir = std::env::temp_dir();
        if cfg!(target_os = "windows") {
            tmp_dir.push(format!("setup-agent-{}.ps1", version));
        } else {
            tmp_dir.push(format!("setup-agent-{}.sh", version));
        }
        tmp_dir
    }

    async fn save_prerelease_script(
        tmp_script: &std::path::PathBuf,
        bytes: &[u8],
    ) -> std::io::Result<()> {
        if cfg!(target_os = "windows") {
            tokio::fs::write(tmp_script, bytes).await
        } else {
            let cmd_str = format!("cat > {}", tmp_script.display());
            let mut cmd = Command::new("sudo");
            cmd.arg("sh")
                .arg("-c")
                .arg(&cmd_str)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped());
            let mut child = cmd.spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = tokio::io::AsyncWriteExt::write_all(&mut stdin, bytes).await;
            }
            let out = child.wait_with_output().await?;
            if out.status.success() {
                Ok(())
            } else {
                Err(std::io::Error::other(
                    String::from_utf8_lossy(&out.stderr).into_owned(),
                ))
            }
        }
    }

    async fn download_prerelease_script(
        version: &str,
        tx: &mpsc::Sender<String>,
    ) -> Option<std::path::PathBuf> {
        let _ = tx
            .send(format!(
                "UPDATE_PROGRESS: [STATUS] Downloading setup script for v{}...",
                version
            ))
            .await;
        let url = Self::get_prerelease_url(version);

        let bytes = match crate::http::fetch_bytes(&url, Duration::from_secs(30)).await {
            Ok(b) => b,
            Err(e) => {
                warn!(error = %e, "Failed to download setup script");
                let _ = tx
                    .send(format!(
                        "UPDATE_PROGRESS: [FAILURE] Failed to download setup script: {e}"
                    ))
                    .await;
                return None;
            }
        };

        let tmp_script = Self::get_prerelease_tmp_script(version);
        info!(script_path = %tmp_script.display(), "Saving setup script to temporary file");

        if let Err(e) = Self::save_prerelease_script(&tmp_script, &bytes).await {
            warn!(error = %e, "Failed to save setup script");
            let _ = tx
                .send(format!(
                    "UPDATE_PROGRESS: [FAILURE] Failed to save setup script: {e}"
                ))
                .await;
            return None;
        }

        if !cfg!(target_os = "windows") {
            let _ = tokio::process::Command::new("chmod")
                .arg("+x")
                .arg(&tmp_script)
                .status()
                .await;
        }

        info!(script = %tmp_script.display(), "Executing prerelease setup script");
        let _ = tx
            .send("UPDATE_PROGRESS: [STATUS] Executing prerelease setup...".to_string())
            .await;
        Some(tmp_script)
    }

    async fn prepare_standard_script(
        paths: &AgentPaths,
        tx: &mpsc::Sender<String>,
    ) -> Option<std::path::PathBuf> {
        info!(script = %paths.update_script.display(), "Executing standard update script");
        if cfg!(target_os = "windows") {
            let tmp_script = std::env::temp_dir().join("adorsys-update.ps1");
            let _ = tx
                .send(
                    "UPDATE_PROGRESS: [STATUS] Downloading fresh Windows update wrapper..."
                        .to_string(),
                )
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
                return None;
            }
            Some(tmp_script)
        } else {
            Some(paths.update_script.clone())
        }
    }

    fn pipe_stdout(
        stdout: tokio::process::ChildStdout,
        tx: mpsc::Sender<String>,
        log_path: std::path::PathBuf,
    ) {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                append_update_log(&log_path, &line).await;
                let _ = tx.send(format!("UPDATE_PROGRESS: {}", line)).await;
            }
        });
    }

    fn pipe_stderr(
        stderr: tokio::process::ChildStderr,
        tx: mpsc::Sender<String>,
        log_path: std::path::PathBuf,
    ) {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                append_update_log(&log_path, &line).await;
                let _ = tx.send(format!("UPDATE_PROGRESS: [ERROR] {}", line)).await;
            }
        });
    }

    fn tail_active_response_log(
        log_path: std::path::PathBuf,
        tx: mpsc::Sender<String>,
        mut kill_rx: tokio::sync::oneshot::Receiver<()>,
    ) {
        if log_path.as_os_str().is_empty() {
            return;
        }
        tokio::spawn(async move {
            let initial_len = tokio::fs::metadata(&log_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0);
            if let Ok(mut file) = tokio::fs::File::open(&log_path).await {
                let _ = file.seek(std::io::SeekFrom::Start(initial_len)).await;
                let mut reader = BufReader::new(file);
                let mut line = String::new();
                loop {
                    tokio::select! {
                        _ = &mut kill_rx => break,
                        res = reader.read_line(&mut line) => {
                            match res {
                                Ok(0) => { tokio::time::sleep(Duration::from_millis(200)).await; }
                                Ok(_) => {
                                    let t = line.trim();
                                    if !t.is_empty() { let _ = tx.send(format!("UPDATE_PROGRESS: {}", t)).await; }
                                    line.clear();
                                }
                                Err(_) => break,
                            }
                        }
                    }
                }
            }
        });
    }

    async fn wait_for_update_command(
        mut child: tokio::process::Child,
        tx: mpsc::Sender<String>,
        log_path: std::path::PathBuf,
        kill_tx: tokio::sync::oneshot::Sender<()>,
    ) {
        match tokio::time::timeout(UPDATE_TIMEOUT, child.wait()).await {
            Ok(Ok(status)) if status.success() => {
                let _ = kill_tx.send(());
                append_update_log(&log_path, "[SUCCESS] Update completed successfully").await;
                tokio::time::sleep(Duration::from_millis(500)).await;
                let _ = tx
                    .send("UPDATE_PROGRESS: [SUCCESS] Update completed successfully".to_string())
                    .await;
            }
            Ok(Ok(status)) => {
                let _ = kill_tx.send(());
                append_update_log(
                    &log_path,
                    &format!(
                        "[FAILURE] Update script exited with code: {:?}",
                        status.code()
                    ),
                )
                .await;
                let _ = tx
                    .send(format!(
                        "UPDATE_PROGRESS: [FAILURE] Update script exited with code: {:?}",
                        status.code()
                    ))
                    .await;
            }
            Ok(Err(e)) => {
                let _ = kill_tx.send(());
                append_update_log(
                    &log_path,
                    &format!("[FAILURE] Failed to wait for update script: {e}"),
                )
                .await;
                let _ = tx
                    .send(format!(
                        "UPDATE_PROGRESS: [FAILURE] Failed to wait for update script: {e}"
                    ))
                    .await;
            }
            Err(_) => {
                let _ = kill_tx.send(());
                #[cfg(unix)]
                if let Some(pid) = child.id() {
                    let _ = unsafe { libc::kill(-(pid as i32), libc::SIGKILL) };
                }
                let _ = child.kill().await;
                let _ = child.wait().await;
                append_update_log(
                    &log_path,
                    &format!(
                        "[FAILURE] Update script timed out after {}s and was terminated",
                        UPDATE_TIMEOUT.as_secs()
                    ),
                )
                .await;
                let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Update script timed out after {}s and was terminated", UPDATE_TIMEOUT.as_secs())).await;
            }
        }
    }

    async fn execute_update_command(
        mut cmd: Command,
        tx: mpsc::Sender<String>,
        paths: Arc<AgentPaths>,
    ) {
        #[cfg(unix)]
        cmd.process_group(0);

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        info!("Spawning update command");

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let error_hint = if cfg!(target_os = "windows") {
                    "check PowerShell"
                } else {
                    "check sudoers"
                };
                let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Failed to start update script ({error_hint}): {e}")).await;
                return;
            }
        };

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let windows_response_log = if cfg!(target_os = "windows") {
            paths.active_response_log.clone()
        } else {
            std::path::PathBuf::new()
        };
        Self::pipe_stdout(stdout, tx.clone(), windows_response_log.clone());
        Self::pipe_stderr(stderr, tx.clone(), windows_response_log.clone());

        let active_response_log = if cfg!(target_os = "windows") {
            std::path::PathBuf::new()
        } else {
            paths.active_response_log.clone()
        };
        let (kill_tx, kill_rx) = tokio::sync::oneshot::channel::<()>();

        Self::tail_active_response_log(active_response_log, tx.clone(), kill_rx);
        Self::wait_for_update_command(child, tx, windows_response_log, kill_tx).await;
    }

    async fn run_update_task(
        paths: Arc<AgentPaths>,
        tx: mpsc::Sender<String>,
        is_prerelease: bool,
        prerelease_version: Option<String>,
        is_manual: bool,
    ) {
        if let Err(e) = tx
            .send("UPDATE_PROGRESS: [STATUS] Starting update process...".to_string())
            .await
        {
            warn!(error = %e, "Failed to send initial progress message");
            return;
        }

        let mut prerelease_tag = None;
        let script_path = if is_prerelease {
            let version = match prerelease_version {
                Some(v) if v != "Unknown" => v,
                _ => {
                    let _ = tx.send("UPDATE_PROGRESS: [FAILURE] Could not determine latest prerelease version".to_string()).await;
                    return;
                }
            };
            prerelease_tag = Some(format!("refs/tags/v{version}"));
            match Self::download_prerelease_script(&version, &tx).await {
                Some(p) => p,
                None => return,
            }
        } else {
            match Self::prepare_standard_script(&paths, &tx).await {
                Some(p) => p,
                None => return,
            }
        };

        let configured_manager = if prerelease_tag.is_some() {
            read_configured_manager()
        } else {
            None
        };

        let cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("powershell.exe");
            c.args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                script_path.to_str().unwrap_or_default(),
            ]);
            if is_manual && prerelease_tag.is_none() {
                c.arg("-Update");
            }
            if is_manual {
                c.env("WAZUH_AGENT_STATUS_UPDATE", "1");
            }
            if let Some(ref tag) = prerelease_tag {
                c.env("WAZUH_AGENT_REPO_REF", tag);
            }
            if let Some(ref manager) = configured_manager {
                c.env("WAZUH_MANAGER", manager);
            }
            c
        } else {
            let is_root = tokio::process::Command::new("id")
                .arg("-u")
                .output()
                .await
                .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
                .unwrap_or(false);
            if is_root {
                let mut c = Command::new(script_path.as_os_str());
                if let Some(ref tag) = prerelease_tag {
                    c.env("WAZUH_AGENT_REPO_REF", tag);
                }
                if let Some(ref manager) = configured_manager {
                    c.env("WAZUH_MANAGER", manager);
                }
                c
            } else {
                let mut c = Command::new("sudo");
                if let Some(ref tag) = prerelease_tag {
                    c.arg(format!("WAZUH_AGENT_REPO_REF={}", tag));
                }
                if let Some(ref manager) = configured_manager {
                    c.arg(format!("WAZUH_MANAGER={}", manager));
                }
                c.arg(script_path.as_os_str());
                c
            }
        };

        Self::execute_update_command(cmd, tx, paths).await;
    }

    pub async fn initiate_update(
        &self,
        is_prerelease: bool,
        is_manual: bool,
    ) -> mpsc::Receiver<String> {
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

        tokio::spawn(Self::run_update_task(
            paths,
            tx,
            is_prerelease,
            prerelease_version,
            is_manual,
        ));

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

    /// Return the current update status.
    ///
    /// **Stable releases** are checked against the plain `version.txt` file.
    /// **Pre-releases** are checked against the full `versions.json` manifest,
    /// which includes `prerelease_version` and `prerelease_test_groups`.
    ///
    /// Results are cached for `config.version_cache_ttl` to avoid hammering
    /// the remote endpoint. The cache is invalidated if the local version
    /// changes (e.g., after an update).
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

        // 2. Fetch the versions.json for pre-release group information
        info!(
            "Fetching pre-release manifest from {}",
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

        // 3. Fetch the plain stable version.txt
        info!(
            "Fetching stable version from {}",
            self.config.stable_version_url
        );
        let stable_version = fetch_plain_version(&self.config.stable_version_url).await;

        match new_info {
            Some(info) => {
                let show_prerelease =
                    crate::version_utils::should_show_prerelease(&info, &current_state.groups);

                // Use the stable version from version.txt; fall back to versions.json's
                // framework.version if the plain fetch failed.
                let effective_stable = stable_version.as_deref().unwrap_or(&info.framework.version);

                let check_update = |name: &str, local_version: &str| {
                    if local_version == "Unknown" || local_version == "Not Installed" {
                        return crate::models::ComponentUpdate {
                            name: name.to_string(),
                            current_version: local_version.to_string(),
                            latest_version: effective_stable.to_string(),
                            state: crate::models::UpdateState::Unknown,
                            can_update: false,
                        };
                    }

                    // Stable check: compare against version.txt
                    let is_outdated = !effective_stable.is_empty()
                        && effective_stable != "Unknown"
                        && crate::version_utils::is_version_higher(effective_stable, local_version);

                    // Pre-release check: compare against versions.json prerelease_version
                    let has_prerelease = !info.framework.prerelease_version.is_empty()
                        && show_prerelease
                        && crate::version_utils::is_version_higher(
                            &info.framework.prerelease_version,
                            local_version,
                        );

                    let (state, latest, can_update) = if is_outdated {
                        (
                            crate::models::UpdateState::Outdated,
                            effective_stable.to_string(),
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
                            effective_stable.to_string(),
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
                // versions.json unavailable — try the plain stable version.txt only
                if let Some(stable) = stable_version {
                    let local = &current_state.tray_version;
                    let is_outdated = local != "Unknown"
                        && local != "Not Installed"
                        && crate::version_utils::is_version_higher(&stable, local);

                    let (state, can_update) = if is_outdated {
                        (crate::models::UpdateState::Outdated, true)
                    } else {
                        (crate::models::UpdateState::UpToDate, false)
                    };

                    UpdateStatus {
                        tray: ComponentUpdate {
                            name: "Wazuh Setup".to_string(),
                            current_version: local.clone(),
                            latest_version: stable,
                            state,
                            can_update,
                        },
                        has_updates: can_update,
                    }
                } else {
                    warn!("Failed to fetch both version manifest and version.txt");
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
}
