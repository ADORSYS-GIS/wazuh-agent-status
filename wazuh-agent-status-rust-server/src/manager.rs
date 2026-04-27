//! Agent state manager — owns the single source of truth for local agent state,
//! broadcasts changes to subscribers, and provides on-demand version checking.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{broadcast, RwLock};
use tokio::time;
use tracing::{info, warn};

use crate::config::{AgentPaths, Config};
use crate::models::{AgentState, ComponentUpdate, LogLine, UpdateStatus, VersionInfo};
use crate::status_provider::StatusProvider;
use crate::version_utils::fetch_version_info;
use tokio::process::Command;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
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

        Self {
            state: Arc::new(RwLock::new(AgentState::default())),
            notifier: tx,
            provider,
            version_cache: RwLock::new(None),
            paths,
            config,
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
                    if self.config.self_healing && new_state.status == crate::models::AgentStatus::Inactive {
                        let now = Instant::now();
                        let should_attempt = match last_healing_attempt {
                            Some(last) => now.duration_since(last) > time::Duration::from_secs(300), // 5-minute cooldown
                            None => true,
                        };

                        if should_attempt {
                            info!("Self-healing: Wazuh agent is inactive. Attempting restart...");
                            last_healing_attempt = Some(now);
                            
                            let control_path = self.paths.wazuh_control.clone();
                            tokio::spawn(async move {
                                let mut cmd = Command::new("sudo");
                                cmd.arg(control_path).arg("restart");
                                
                                match cmd.output().await {
                                    Ok(o) => {
                                        if o.status.success() {
                                            info!("Self-healing: Restart command executed successfully");
                                        } else {
                                            warn!("Self-healing: Restart command failed with exit code {}: {}", 
                                                o.status.code().unwrap_or(-1),
                                                String::from_utf8_lossy(&o.stderr));
                                        }
                                    },
                                    Err(e) => warn!("Self-healing: Failed to spawn restart command: {e}"),
                                }
                            });
                        }
                    } else if new_state.status == crate::models::AgentStatus::Active {
                        // Just broadcast state; don't reset healing clock to maintain strict cooldown
                    }

                    let mut final_state = new_state;
                    final_state.self_healing_enabled = self.config.self_healing;

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
            Some(status.wazuh.latest_version)
        } else {
            None
        };

        tokio::spawn(async move {
            let _ = tx.send("UPDATE_PROGRESS: [STATUS] Starting update process...".to_string()).await;

            let mut cmd = Command::new("sudo");
            if is_prerelease {
                let version = match prerelease_version {
                    Some(v) if v != "Unknown" => v,
                    _ => {
                        let _ = tx.send("UPDATE_PROGRESS: [FAILURE] Could not determine latest prerelease version".to_string()).await;
                        return;
                    }
                };

                let _ = tx.send(format!("UPDATE_PROGRESS: [STATUS] Downloading setup script for v{}...", version)).await;
                let url = format!("https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v{}/scripts/setup-agent.sh", version);
                
                match crate::http::fetch_bytes(&url, Duration::from_secs(30)).await {
                    Ok(bytes) => {
                        let tmp_script = format!("/tmp/setup-agent-{}.sh", version);
                        if let Err(e) = std::fs::write(&tmp_script, bytes) {
                            let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Failed to save setup script: {e}")).await;
                            return;
                        }
                        let _ = std::process::Command::new("chmod").arg("+x").arg(&tmp_script).status();
                        
                        let _ = tx.send("UPDATE_PROGRESS: [STATUS] Executing prerelease setup...".to_string()).await;
                        cmd.arg(tmp_script);
                    }
                    Err(e) => {
                        let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Failed to download setup script: {e}")).await;
                        return;
                    }
                }
            } else {
                cmd.arg(&paths.update_script);
            }

            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    let stdout = child.stdout.take().unwrap();
                    let stderr = child.stderr.take().unwrap();
                    let tx_clone = tx.clone();

                    // Pipe stdout
                    tokio::spawn(async move {
                        let mut reader = BufReader::new(stdout).lines();
                        while let Ok(Some(line)) = reader.next_line().await {
                            let _ = tx_clone.send(format!("UPDATE_PROGRESS: {}", line)).await;
                        }
                    });

                    // Pipe stderr
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        let mut reader = BufReader::new(stderr).lines();
                        while let Ok(Some(line)) = reader.next_line().await {
                            let _ = tx_clone.send(format!("UPDATE_PROGRESS: [ERROR] {}", line)).await;
                        }
                    });

                    match child.wait().await {
                        Ok(status) if status.success() => {
                            let _ = tx.send("UPDATE_PROGRESS: [SUCCESS] Update completed successfully".to_string()).await;
                        }
                        Ok(status) => {
                            let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Update script exited with code: {:?}", status.code())).await;
                        }
                        Err(e) => {
                            let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Failed to wait for update script: {e}")).await;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Failed to start update script (check sudoers): {e}")).await;
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
            let file = match tokio::fs::File::open(&log_path).await {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.send(LogLine::from_raw(format!("[ERROR] Cannot open log file: {e}"))).await;
                    return;
                }
            };

            let mut reader = BufReader::new(file);
            // Jump to end so we only stream *new* lines (real-time tail).
            if let Err(e) = reader.seek(std::io::SeekFrom::End(0)).await {
                let _ = tx.send(LogLine::from_raw(format!("[ERROR] Cannot seek log file: {e}"))).await;
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
                        let _ = tx.send(LogLine::from_raw(format!("[ERROR] Failed to read log line: {e}"))).await;
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
    /// the remote manifest endpoint.
    pub async fn get_version_status(&self) -> UpdateStatus {
        let now = Instant::now();
        let current_state = self.get_state().await;

        // 1. Try to return fresh cached value
        {
            let cache = self.version_cache.read().await;
            if let Some(c) = &*cache {
                if now.duration_since(c.fetched_at) < self.config.version_cache_ttl {
                    return c.status.clone();
                }
            }
        }

        // 2. Fetch fresh data
        info!("Fetching fresh version manifest from {}", self.config.version_url);
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
                let wazuh_v = info.components.get("wazuh-agent");
                let wazuh_update = compute_component_update(
                    "Wazuh Agent",
                    &current_state.version,
                    &current_state.groups,
                    wazuh_v.map(|v| v.version.as_str()).unwrap_or("Unknown"),
                    wazuh_v.map(|v| v.prerelease_version.as_str()).unwrap_or(""),
                    &info.prerelease_test_groups,
                );

                let tray_update = compute_component_update(
                    "Tray App",
                    &current_state.tray_version,
                    &current_state.groups,
                    &info.framework.version,
                    &info.framework.prerelease_version,
                    &[], // Tray uses global framework version
                );

                let has_updates = wazuh_update.can_update || tray_update.can_update;
                let status = UpdateStatus {
                    wazuh: wazuh_update,
                    tray: tray_update,
                    has_updates,
                };

                let mut cache = self.version_cache.write().await;
                *cache = Some(VersionCache {
                    status: status.clone(),
                    info,
                    fetched_at: if is_fallback {
                        self.version_cache.read().await.as_ref().map(|c| c.fetched_at).unwrap_or(now)
                    } else {
                        now
                    },
                });
                status
            }
            None => {
                warn!("Failed to fetch remote version manifest and no cache available");
                UpdateStatus {
                    wazuh: ComponentUpdate {
                        name: "Wazuh Agent".to_string(),
                        current_version: current_state.version,
                        latest_version: "Unknown".to_string(),
                        state: crate::models::UpdateState::Unknown,
                        can_update: false,
                    },
                    tray: ComponentUpdate {
                        name: "Tray App".to_string(),
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

fn compute_component_update(
    name: &str,
    local_version: &str,
    agent_groups: &[String],
    online_version: &str,
    online_prerelease: &str,
    prerelease_groups: &[String],
) -> crate::models::ComponentUpdate {
    if local_version == "Unknown" || local_version == "Not Installed" {
        return crate::models::ComponentUpdate {
            name: name.to_string(),
            current_version: local_version.to_string(),
            latest_version: online_version.to_string(),
            state: crate::models::UpdateState::Unknown,
            can_update: false,
        };
    }

    let is_outdated = !online_version.is_empty()
        && online_version != "Unknown"
        && crate::version_utils::is_version_higher(online_version, local_version);

    let has_prerelease = !online_prerelease.is_empty()
        && should_show_prerelease_for_component(prerelease_groups, agent_groups)
        && crate::version_utils::is_version_higher(online_prerelease, local_version);

    let (state, latest, can_update) = if is_outdated {
        (crate::models::UpdateState::Outdated, online_version.to_string(), true)
    } else if has_prerelease {
        (crate::models::UpdateState::PrereleaseAvailable, online_prerelease.to_string(), true)
    } else {
        (crate::models::UpdateState::UpToDate, online_version.to_string(), false)
    };

    crate::models::ComponentUpdate {
        name: name.to_string(),
        current_version: local_version.to_string(),
        latest_version: latest,
        state,
        can_update,
    }
}

fn should_show_prerelease_for_component(manifest_groups: &[String], agent_groups: &[String]) -> bool {
    if manifest_groups.is_empty() || agent_groups.is_empty() {
        return false;
    }
    agent_groups.iter().any(|ag| {
        manifest_groups.iter().any(|tg| ag.eq_ignore_ascii_case(tg))
    })
}
