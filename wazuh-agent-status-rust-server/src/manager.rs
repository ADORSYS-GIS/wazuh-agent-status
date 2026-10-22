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

        tokio::spawn(run_update_task(
            paths,
            tx,
            is_prerelease,
            is_manual,
            prerelease_version,
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

// ── Update Helpers ────────────────────────────────────────────────────────────

async fn save_setup_script(tmp_script: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
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
            let _ = stdin.write_all(bytes).await;
            drop(stdin);
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

    let filename = if cfg!(target_os = "windows") {
        format!("setup-agent-{}.ps1", version)
    } else {
        format!("setup-agent-{}.sh", version)
    };
    let tmp_script = std::env::temp_dir().join(filename);

    info!(script_path = %tmp_script.display(), "Saving setup script to temporary file");
    if let Err(e) = save_setup_script(&tmp_script, &bytes).await {
        warn!(error = %e, "Failed to save setup script");
        let _ = tx
            .send(format!(
                "UPDATE_PROGRESS: [FAILURE] Failed to save setup script: {e}"
            ))
            .await;
        return None;
    }

    // Make executable on Unix
    if !cfg!(target_os = "windows") {
        let _ = Command::new("chmod")
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
                "UPDATE_PROGRESS: [STATUS] Downloading fresh Windows update wrapper...".to_string(),
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

async fn build_update_command(
    script_path: &std::path::Path,
    is_manual: bool,
    prerelease_tag: Option<&str>,
    configured_manager: Option<&str>,
) -> Command {
    let mut cmd = if cfg!(target_os = "windows") {
        let script_str = script_path.to_str().unwrap_or_default();
        info!("Running PowerShell script directly");
        let mut c = Command::new("powershell.exe");
        c.args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            script_str,
        ]);
        if is_manual && prerelease_tag.is_none() {
            c.arg("-Update");
        }
        if is_manual {
            c.env("WAZUH_AGENT_STATUS_UPDATE", "1");
        }
        if let Some(tag) = prerelease_tag {
            c.env("WAZUH_AGENT_REPO_REF", tag);
        }
        if let Some(manager) = configured_manager {
            c.env("WAZUH_MANAGER", manager);
        }
        c
    } else {
        let is_root = Command::new("id")
            .arg("-u")
            .output()
            .await
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
            .unwrap_or(false);

        if is_root {
            info!("Running as root — executing update script directly");
            let mut c = Command::new(script_path.as_os_str());
            if let Some(tag) = prerelease_tag {
                c.env("WAZUH_AGENT_REPO_REF", tag);
            }
            if let Some(manager) = configured_manager {
                c.env("WAZUH_MANAGER", manager);
            }
            c
        } else {
            info!("Running as non-root — using sudo for update script");
            let mut c = Command::new("sudo");
            if let Some(tag) = prerelease_tag {
                c.arg(format!("WAZUH_AGENT_REPO_REF={}", tag));
            }
            if let Some(manager) = configured_manager {
                c.arg(format!("WAZUH_MANAGER={}", manager));
            }
            c.arg(script_path.as_os_str());
            c
        }
    };

    #[cfg(unix)]
    cmd.process_group(0);

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    cmd
}

fn spawn_log_pipe<R>(
    reader: R,
    tx: mpsc::Sender<String>,
    log_path: std::path::PathBuf,
    is_err: bool,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if is_err {
                warn!(line = %line, "Update stderr");
                append_update_log(&log_path, &line).await;
                let _ = tx.send(format!("UPDATE_PROGRESS: [ERROR] {}", line)).await;
            } else {
                info!(line = %line, "Update stdout");
                append_update_log(&log_path, &line).await;
                let _ = tx.send(format!("UPDATE_PROGRESS: {}", line)).await;
            }
        }
    });
}

fn spawn_active_response_tailer(
    log_path: std::path::PathBuf,
    tx: mpsc::Sender<String>,
) -> Option<tokio::sync::oneshot::Sender<()>> {
    if log_path.as_os_str().is_empty() {
        return None;
    }

    let (kill_tx, mut kill_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let initial_len = match tokio::fs::metadata(&log_path).await {
            Ok(m) => m.len(),
            Err(_) => 0,
        };

        let mut file = match tokio::fs::File::open(&log_path).await {
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
                                let _ = tx.send(format!("UPDATE_PROGRESS: {}", t)).await;
                            }
                            line.clear();
                        }
                        Err(_) => break,
                    }
                }
            }
        }
    });

    Some(kill_tx)
}

async fn wait_for_update_child(
    mut child: tokio::process::Child,
    tx: &mpsc::Sender<String>,
    windows_response_log: &std::path::Path,
    kill_tx: Option<tokio::sync::oneshot::Sender<()>>,
) {
    let res = tokio::time::timeout(UPDATE_TIMEOUT, child.wait()).await;
    if let Some(kill) = kill_tx {
        let _ = kill.send(());
    }

    match res {
        Ok(Ok(status)) if status.success() => {
            info!(exit_code = ?status.code(), "Update script completed successfully");
            append_update_log(
                windows_response_log,
                "[SUCCESS] Update completed successfully",
            )
            .await;
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = tx
                .send("UPDATE_PROGRESS: [SUCCESS] Update completed successfully".to_string())
                .await;
        }
        Ok(Ok(status)) => {
            warn!(exit_code = ?status.code(), "Update script failed");
            append_update_log(
                windows_response_log,
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
            warn!(error = %e, "Failed to wait for update script");
            append_update_log(
                windows_response_log,
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
            debug!(
                seconds = UPDATE_TIMEOUT.as_secs(),
                "Update script exceeded hard timeout; killing and reaping"
            );
            #[cfg(unix)]
            if let Some(pid) = child.id() {
                let _ = unsafe { libc::kill(-(pid as i32), libc::SIGKILL) };
            }
            let _ = child.kill().await;
            let _ = child.wait().await;
            warn!(
                seconds = UPDATE_TIMEOUT.as_secs(),
                "Update script timed out; terminated"
            );
            append_update_log(
                windows_response_log,
                &format!(
                    "[FAILURE] Update script timed out after {}s and was terminated",
                    UPDATE_TIMEOUT.as_secs()
                ),
            )
            .await;
            let _ = tx
                .send(format!(
                    "UPDATE_PROGRESS: [FAILURE] Update script timed out after {}s and was terminated",
                    UPDATE_TIMEOUT.as_secs()
                ))
                .await;
        }
    }
}

async fn run_update_task(
    paths: Arc<AgentPaths>,
    tx: mpsc::Sender<String>,
    is_prerelease: bool,
    is_manual: bool,
    prerelease_version: Option<String>,
) {
    info!("Update task started, sending initial progress message");
    if let Err(e) = tx
        .send("UPDATE_PROGRESS: [STATUS] Starting update process...".to_string())
        .await
    {
        warn!(error = %e, "Failed to send initial progress message");
        return;
    }
    info!("Initial progress message sent successfully");

    let (script_path, prerelease_tag) = if is_prerelease {
        let version = match prerelease_version {
            Some(v) if v != "Unknown" => v,
            _ => {
                warn!("Could not determine latest prerelease version");
                let _ = tx
                    .send(
                        "UPDATE_PROGRESS: [FAILURE] Could not determine latest prerelease version"
                            .to_string(),
                    )
                    .await;
                return;
            }
        };

        info!(version = %version, "Processing prerelease update");
        let tag = format!("refs/tags/v{version}");
        let path = match download_prerelease_script(&version, &tx).await {
            Some(p) => p,
            None => return,
        };
        (path, Some(tag))
    } else {
        let path = match prepare_standard_script(&paths, &tx).await {
            Some(p) => p,
            None => return,
        };
        (path, None)
    };

    let configured_manager = if prerelease_tag.is_some() {
        read_configured_manager()
    } else {
        None
    };

    let mut cmd = build_update_command(
        &script_path,
        is_manual,
        prerelease_tag.as_deref(),
        configured_manager.as_deref(),
    )
    .await;

    debug!(script = %script_path.display(), "Update command prepared");
    info!("Spawning update command");

    match cmd.spawn() {
        Ok(mut child) => {
            debug!(pid = child.id(), "Update script spawned");
            info!("Update command spawned successfully");

            let windows_response_log = if cfg!(target_os = "windows") {
                paths.active_response_log.clone()
            } else {
                std::path::PathBuf::new()
            };

            if let Some(stdout) = child.stdout.take() {
                spawn_log_pipe(stdout, tx.clone(), windows_response_log.clone(), false);
            }

            if let Some(stderr) = child.stderr.take() {
                spawn_log_pipe(stderr, tx.clone(), windows_response_log.clone(), true);
            }

            let active_response_log = if cfg!(target_os = "windows") {
                std::path::PathBuf::new()
            } else {
                paths.active_response_log.clone()
            };

            let kill_tx = spawn_active_response_tailer(active_response_log, tx.clone());
            wait_for_update_child(child, &tx, &windows_response_log, kill_tx).await;
        }
        Err(e) => {
            warn!(error = %e, "Failed to spawn update command");
            let error_hint = if cfg!(target_os = "windows") {
                "check that PowerShell and the script path are available"
            } else {
                "check sudoers configuration"
            };
            let _ = tx
                .send(format!(
                    "UPDATE_PROGRESS: [FAILURE] Failed to start update script ({error_hint}): {e}"
                ))
                .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    struct TestProvider {
        state: AgentState,
    }

    impl StatusProvider for TestProvider {
        fn get_agent_status(
            &self,
        ) -> Result<crate::models::AgentStatus, crate::errors::ServerError> {
            Ok(self.state.status.clone())
        }
        fn get_connection_status(
            &self,
        ) -> Result<crate::models::ConnectionStatus, crate::errors::ServerError> {
            Ok(self.state.connection.clone())
        }
        fn get_agent_version(&self) -> Result<String, crate::errors::ServerError> {
            Ok(self.state.version.clone())
        }
        fn get_tray_version(&self) -> Result<String, crate::errors::ServerError> {
            Ok(self.state.tray_version.clone())
        }
        fn get_agent_groups(&self) -> Result<Vec<String>, crate::errors::ServerError> {
            Ok(self.state.groups.clone())
        }
        fn get_system_metrics(
            &self,
        ) -> Result<crate::models::SystemMetrics, crate::errors::ServerError> {
            Ok(self.state.metrics.clone())
        }
        fn get_partial_state(&self) -> Result<AgentState, crate::errors::ServerError> {
            Ok(self.state.clone())
        }
    }

    fn make_test_paths(dir: &std::path::Path) -> AgentPaths {
        AgentPaths {
            merged_mg: dir.join("merged.mg"),
            state_file: dir.join("wazuh-agentd.state"),
            version_json: dir.join("version.json"),
            version_file: dir.join("version.txt"),
            pid_file: dir.join("wazuh-agentd.pid"),
            update_script: dir.join("update.sh"),
            wazuh_control: dir.join("wazuh-control"),
            ossec_log: dir.join("ossec.log"),
            client_keys: dir.join("client.keys"),
            active_response_log: dir.join("active-responses.log"),
        }
    }

    #[tokio::test]
    async fn test_append_update_log() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("sub").join("test_update.log");

        // Empty path should be safely ignored
        append_update_log(std::path::Path::new(""), "ignored").await;

        append_update_log(&log_path, "line 1").await;
        append_update_log(&log_path, "line 2").await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert_eq!(content, "line 1\nline 2\n");
    }

    #[test]
    fn test_read_client_keys_variations() {
        let dir = tempdir().unwrap();
        let paths = make_test_paths(dir.path());

        // 1. Missing file
        let (id, name, key) = AgentManager::read_client_keys(&paths);
        assert_eq!((id.as_str(), name.as_str(), key.as_str()), ("", "", ""));

        // 2. Empty file
        std::fs::write(&paths.client_keys, "").unwrap();
        let (id, name, key) = AgentManager::read_client_keys(&paths);
        assert_eq!((id.as_str(), name.as_str(), key.as_str()), ("", "", ""));

        // 3. Single token (malformed)
        std::fs::write(&paths.client_keys, "onlyone\n").unwrap();
        let (id, name, key) = AgentManager::read_client_keys(&paths);
        assert_eq!((id.as_str(), name.as_str(), key.as_str()), ("", "", ""));

        // 4. Two tokens
        std::fs::write(&paths.client_keys, "001 agent-linux\n").unwrap();
        let (id, name, key) = AgentManager::read_client_keys(&paths);
        assert_eq!(
            (id.as_str(), name.as_str(), key.as_str()),
            ("001", "agent-linux", "")
        );

        // 5. Full 4+ tokens: id name ip key
        std::fs::write(
            &paths.client_keys,
            "042 production-srv any 1234567890abcdef extra-data\n",
        )
        .unwrap();
        let (id, name, key) = AgentManager::read_client_keys(&paths);
        assert_eq!(
            (id.as_str(), name.as_str(), key.as_str()),
            ("042", "production-srv", "1234567890abcdef")
        );
    }

    #[tokio::test]
    async fn test_manager_initialization_and_state() {
        let dir = tempdir().unwrap();
        let paths = make_test_paths(dir.path());
        std::fs::write(&paths.client_keys, "007 bond any secret777\n").unwrap();

        let config = Arc::new(Config {
            self_healing: false,
            ..Config::default()
        });

        let provider = Box::new(TestProvider {
            state: AgentState {
                status: crate::models::AgentStatus::Active,
                connection: crate::models::ConnectionStatus::Connected,
                version: "4.8.0".to_string(),
                tray_version: "1.2.0".to_string(),
                groups: vec!["production".to_string()],
                metrics: crate::models::SystemMetrics::default(),
                self_healing_enabled: false,
                agent_id: "007".to_string(),
                agent_name: "bond".to_string(),
                agent_key: "secret777".to_string(),
            },
        });

        let manager = AgentManager::new_custom(Arc::clone(&config), Arc::new(paths), provider);

        assert!(!manager.config().self_healing);
        let state = manager.get_state().await;
        assert_eq!(state.agent_id, "007");
        assert_eq!(state.agent_name, "bond");
        assert_eq!(state.agent_key, "secret777");

        let mut sub = manager.subscribe();
        assert!(sub.try_recv().is_err()); // No messages yet
    }

    #[tokio::test]
    async fn test_stream_logs_missing_and_existing() {
        let dir = tempdir().unwrap();
        let paths = make_test_paths(dir.path());
        let config = Arc::new(Config {
            self_healing: false,
            ..Config::default()
        });

        let provider = Box::new(TestProvider {
            state: AgentState::default(),
        });
        let manager = AgentManager::new_custom(config, Arc::new(paths.clone()), provider);

        // 1. Missing file
        let mut rx = manager.stream_logs().await;
        let line = rx.recv().await.unwrap();
        assert!(line.raw.contains("[ERROR] Log file not found"));

        // 2. Existing file with historical lines
        let mut file = std::fs::File::create(&paths.ossec_log).unwrap();
        writeln!(file, "2026/10/21 12:00:00 ossec-agent: INFO: Starting").unwrap();
        writeln!(file, "2026/10/21 12:00:01 ossec-agent: INFO: Connected").unwrap();
        drop(file);

        let mut rx2 = manager.stream_logs().await;
        let line1 = rx2.recv().await.unwrap();
        assert!(line1.raw.contains("Starting"));
        let line2 = rx2.recv().await.unwrap();
        assert!(line2.raw.contains("Connected"));
    }

    #[tokio::test]
    async fn test_check_and_auto_update_in_progress_guard() {
        let dir = tempdir().unwrap();
        let paths = make_test_paths(dir.path());
        let config = Arc::new(Config {
            self_healing: false,
            ..Config::default()
        });

        let provider = Box::new(TestProvider {
            state: AgentState::default(),
        });
        let manager = AgentManager::new_custom(config, Arc::new(paths), provider);

        // Simulate update already running
        manager.update_in_progress.store(true, Ordering::SeqCst);
        manager.check_and_auto_update().await;
        // If guard works, flag remains true
        assert!(manager.update_in_progress.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_version_cache_fresh_hit() {
        let dir = tempdir().unwrap();
        let paths = make_test_paths(dir.path());
        let config = Arc::new(Config {
            version_cache_ttl: Duration::from_secs(3600),
            self_healing: false,
            ..Config::default()
        });

        let provider = Box::new(TestProvider {
            state: AgentState {
                tray_version: "2.0.0".to_string(),
                ..AgentState::default()
            },
        });
        let manager = AgentManager::new_custom(config, Arc::new(paths), provider);
        manager.state.write().await.tray_version = "2.0.0".to_string();

        // Pre-populate cache
        let cached_status = UpdateStatus {
            tray: ComponentUpdate {
                name: "Wazuh Setup".to_string(),
                current_version: "2.0.0".to_string(),
                latest_version: "2.0.0".to_string(),
                state: crate::models::UpdateState::UpToDate,
                can_update: false,
            },
            has_updates: false,
        };

        {
            let mut cache = manager.version_cache.write().await;
            *cache = Some(VersionCache {
                status: cached_status.clone(),
                info: VersionInfo {
                    framework: crate::models::FrameworkVersion {
                        version: "2.0.0".to_string(),
                        prerelease_version: String::new(),
                    },
                    prerelease_test_groups: vec![],
                },
                fetched_at: Instant::now(),
            });
        }

        // Call get_version_status: should hit the fresh cache directly
        let status = manager.get_version_status().await;
        assert_eq!(status.tray.current_version, "2.0.0");
        assert_eq!(status.tray.state, crate::models::UpdateState::UpToDate);
        assert!(!status.has_updates);
    }

    #[test]
    fn test_read_configured_manager_returns_option() {
        let _ = read_configured_manager();
    }

    #[tokio::test]
    async fn test_get_version_status_both_endpoints_fail() {
        let dir = tempdir().unwrap();
        let paths = make_test_paths(dir.path());
        let config = Arc::new(Config {
            version_url: "http://127.0.0.1:9/invalid".to_string(),
            stable_version_url: "http://127.0.0.1:9/invalid".to_string(),
            self_healing: false,
            ..Config::default()
        });

        let provider = Box::new(TestProvider {
            state: AgentState {
                tray_version: "1.0.0".to_string(),
                ..AgentState::default()
            },
        });
        let manager = AgentManager::new_custom(config, Arc::new(paths), provider);
        manager.state.write().await.tray_version = "1.0.0".to_string();

        let status = manager.get_version_status().await;
        assert_eq!(status.tray.latest_version, "Unknown");
        assert_eq!(status.tray.current_version, "1.0.0");
        assert_eq!(status.tray.state, crate::models::UpdateState::Unknown);
        assert!(!status.has_updates);
    }

    // ── Mock HTTP helpers ─────────────────────────────────────────────────────

    /// Minimal HTTP/1.1 server serving `/versions.json` and `/version.txt`.
    /// A `None` body yields a 404 for that route.
    async fn spawn_mock_server(
        manifest: Option<&'static str>,
        stable: Option<&'static str>,
    ) -> String {
        use tokio::io::AsyncReadExt;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 4096];
                    let n = sock.read(&mut buf).await.unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let body = if req.starts_with("GET /versions.json") {
                        manifest
                    } else if req.starts_with("GET /version.txt") {
                        stable
                    } else {
                        None
                    };
                    let resp = match body {
                        Some(b) => format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            b.len(),
                            b
                        ),
                        None => "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                            .to_string(),
                    };
                    let _ = sock.write_all(resp.as_bytes()).await;
                    let _ = sock.shutdown().await;
                });
            }
        });
        format!("http://{addr}")
    }

    fn manager_with(
        base: &str,
        tray_version: &str,
        groups: Vec<String>,
        dir: &std::path::Path,
    ) -> AgentManager {
        let config = Arc::new(Config {
            version_url: format!("{base}/versions.json"),
            stable_version_url: format!("{base}/version.txt"),
            version_cache_ttl: Duration::from_secs(3600),
            self_healing: false,
            ..Config::default()
        });
        let state = AgentState {
            tray_version: tray_version.to_string(),
            groups,
            ..AgentState::default()
        };
        let provider = Box::new(TestProvider {
            state: state.clone(),
        });
        AgentManager::new_custom(config, Arc::new(make_test_paths(dir)), provider)
    }

    async fn set_local(manager: &AgentManager, tray_version: &str, groups: Vec<String>) {
        let mut s = manager.state.write().await;
        s.tray_version = tray_version.to_string();
        s.groups = groups;
    }

    // ── get_version_status branches ──────────────────────────────────────────

    #[tokio::test]
    async fn test_get_version_status_manifest_outdated() {
        let dir = tempdir().unwrap();
        let base = spawn_mock_server(
            Some(r#"{"framework":{"version":"2.0.0","prerelease_version":""}}"#),
            Some("2.0.0\n"),
        )
        .await;
        let manager = manager_with(&base, "1.0.0", vec![], dir.path());
        set_local(&manager, "1.0.0", vec![]).await;

        let status = manager.get_version_status().await;
        assert_eq!(status.tray.state, crate::models::UpdateState::Outdated);
        assert_eq!(status.tray.latest_version, "2.0.0");
        assert!(status.has_updates);

        // Second call hits the fresh cache
        let cached = manager.get_version_status().await;
        assert_eq!(cached.tray.state, crate::models::UpdateState::Outdated);

        // Local version changes -> cache invalidated and recomputed
        set_local(&manager, "2.0.0", vec![]).await;
        let refreshed = manager.get_version_status().await;
        assert_eq!(refreshed.tray.state, crate::models::UpdateState::UpToDate);
        assert_eq!(refreshed.tray.current_version, "2.0.0");
        assert!(!refreshed.has_updates);
    }

    #[tokio::test]
    async fn test_get_version_status_manifest_prerelease() {
        let dir = tempdir().unwrap();
        let base = spawn_mock_server(
            Some(
                r#"{"framework":{"version":"1.0.0","prerelease_version":"1.1.0-rc1"},"prerelease_test_groups":["beta"]}"#,
            ),
            Some("1.0.0"),
        )
        .await;
        let groups = vec!["Beta".to_string()];
        let manager = manager_with(&base, "1.0.0", groups.clone(), dir.path());
        set_local(&manager, "1.0.0", groups).await;

        let status = manager.get_version_status().await;
        assert_eq!(
            status.tray.state,
            crate::models::UpdateState::PrereleaseAvailable
        );
        assert_eq!(status.tray.latest_version, "1.1.0-rc1");
        assert!(status.has_updates);
    }

    #[tokio::test]
    async fn test_get_version_status_manifest_unknown_local() {
        let dir = tempdir().unwrap();
        // version.txt missing -> falls back to manifest framework.version
        let base = spawn_mock_server(Some(r#"{"framework":{"version":"3.0.0"}}"#), None).await;
        let manager = manager_with(&base, "Unknown", vec![], dir.path());
        set_local(&manager, "Unknown", vec![]).await;

        let status = manager.get_version_status().await;
        assert_eq!(status.tray.state, crate::models::UpdateState::Unknown);
        assert_eq!(status.tray.latest_version, "3.0.0");
        assert!(!status.has_updates);
    }

    #[tokio::test]
    async fn test_get_version_status_stable_only() {
        let dir = tempdir().unwrap();
        // versions.json 404 (no cache, so no fallback) -> plain version.txt path
        let base = spawn_mock_server(None, Some("3.0.0")).await;
        let manager = manager_with(&base, "2.0.0", vec![], dir.path());

        set_local(&manager, "2.0.0", vec![]).await;
        let status = manager.get_version_status().await;
        assert_eq!(status.tray.state, crate::models::UpdateState::Outdated);
        assert_eq!(status.tray.latest_version, "3.0.0");
        assert!(status.has_updates);

        set_local(&manager, "3.0.0", vec![]).await;
        let status = manager.get_version_status().await;
        assert_eq!(status.tray.state, crate::models::UpdateState::UpToDate);
        assert!(!status.has_updates);
    }

    #[tokio::test]
    async fn test_get_version_status_invalid_manifest_and_empty_stable() {
        let dir = tempdir().unwrap();
        // Malformed JSON manifest + blank version.txt -> both treated as unavailable
        let base = spawn_mock_server(Some("not json"), Some("  \n")).await;
        let manager = manager_with(&base, "1.0.0", vec![], dir.path());
        set_local(&manager, "1.0.0", vec![]).await;

        let status = manager.get_version_status().await;
        assert_eq!(status.tray.state, crate::models::UpdateState::Unknown);
        assert_eq!(status.tray.latest_version, "Unknown");
        assert!(!status.has_updates);
    }

    #[tokio::test]
    async fn test_check_and_auto_update_up_to_date() {
        let dir = tempdir().unwrap();
        let base =
            spawn_mock_server(Some(r#"{"framework":{"version":"1.0.0"}}"#), Some("1.0.0")).await;
        let manager = manager_with(&base, "1.0.0", vec![], dir.path());
        set_local(&manager, "1.0.0", vec![]).await;

        manager.check_and_auto_update().await;
        // Guard must be released after returning
        assert!(!manager.update_in_progress.load(Ordering::SeqCst));
    }

    // ── Polling loop ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_start_polling_broadcasts_state() {
        let dir = tempdir().unwrap();
        let base =
            spawn_mock_server(Some(r#"{"framework":{"version":"1.2.0"}}"#), Some("1.2.0")).await;
        let config = Arc::new(Config {
            version_url: format!("{base}/versions.json"),
            stable_version_url: format!("{base}/version.txt"),
            poll_interval: Duration::from_millis(50),
            auto_update_check_interval: Duration::from_secs(3600),
            self_healing: false,
            ..Config::default()
        });
        let provider = Box::new(TestProvider {
            state: AgentState {
                status: crate::models::AgentStatus::Active,
                connection: crate::models::ConnectionStatus::Connected,
                version: "4.8.0".to_string(),
                tray_version: "1.2.0".to_string(),
                ..AgentState::default()
            },
        });
        let manager =
            AgentManager::new_custom(config, Arc::new(make_test_paths(dir.path())), provider);
        let mut sub = manager.subscribe();

        let _ = tokio::time::timeout(Duration::from_millis(400), manager.start_polling()).await;

        let update = sub
            .try_recv()
            .expect("polling should broadcast a state change");
        assert_eq!(update.version, "4.8.0");
        assert_eq!(update.tray_version, "1.2.0");
        assert!(!update.self_healing_enabled);
        assert_eq!(manager.get_state().await.version, "4.8.0");
        assert!(!manager.update_in_progress.load(Ordering::SeqCst));
    }

    // ── Log streaming ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_stream_logs_history_cap_and_live_tail() {
        let dir = tempdir().unwrap();
        let paths = make_test_paths(dir.path());
        let mut file = std::fs::File::create(&paths.ossec_log).unwrap();
        for i in 0..60 {
            writeln!(file, "history line {i}").unwrap();
        }
        drop(file);

        let provider = Box::new(TestProvider {
            state: AgentState::default(),
        });
        let manager = AgentManager::new_custom(
            Arc::new(Config::default()),
            Arc::new(paths.clone()),
            provider,
        );

        let mut rx = manager.stream_logs().await;
        // Only the last 50 lines are replayed
        let first = rx.recv().await.unwrap();
        assert!(first.raw.contains("history line 10"));
        for _ in 0..49 {
            rx.recv().await.unwrap();
        }

        // Give the tailer time to seek to EOF, then append a live line
        tokio::time::sleep(Duration::from_millis(200)).await;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&paths.ossec_log)
            .unwrap();
        writeln!(file, "live line").unwrap();
        drop(file);

        let live = tokio::time::timeout(Duration::from_secs(3), rx.recv())
            .await
            .expect("live line should arrive")
            .unwrap();
        assert!(live.raw.contains("live line"));
    }

    #[tokio::test]
    async fn test_update_helpers() {
        let (tx, mut rx) = mpsc::channel(10);
        let dir = tempdir().unwrap();
        let paths = make_test_paths(dir.path());

        let script = prepare_standard_script(&paths, &tx).await;
        assert!(script.is_some());

        let p = std::path::Path::new("/bin/true");
        let _cmd1 = build_update_command(p, false, None, None).await;
        let _cmd2 = build_update_command(p, true, Some("v1.2.3"), Some("10.0.0.1")).await;

        let log_file = dir.path().join("update.log");
        let cursor = std::io::Cursor::new(b"test output line\n");
        spawn_log_pipe(cursor, tx.clone(), log_file.clone(), false);
        let msg = rx.recv().await.unwrap();
        assert!(msg.contains("test output line"));

        let cursor_err = std::io::Cursor::new(b"test err line\n");
        spawn_log_pipe(cursor_err, tx.clone(), log_file, true);
        let msg_err = rx.recv().await.unwrap();
        assert!(msg_err.contains("test err line"));

        let tailer = spawn_active_response_tailer(std::path::PathBuf::new(), tx);
        assert!(tailer.is_none());
    }
}
