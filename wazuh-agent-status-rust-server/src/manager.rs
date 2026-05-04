//! Agent state manager — owns the single source of truth for local agent state,
//! broadcasts changes to subscribers, and provides on-demand version checking.

use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{broadcast, RwLock};
use tokio::time;
use tracing::{info, warn};

use crate::config::{AgentPaths, Config};
use crate::models::{AgentState, ComponentUpdate, UpdateStatus, VersionInfo};
use crate::status_provider::StatusProvider;
use crate::version_utils::fetch_version_info;

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
        _paths: Arc<AgentPaths>,
        provider: Box<dyn StatusProvider>,
    ) -> Self {
        let (tx, _) = broadcast::channel(128);

        Self {
            state: Arc::new(RwLock::new(AgentState::default())),
            notifier: tx,
            provider,
            version_cache: RwLock::new(None),
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
        loop {
            ticker.tick().await;
            match self.provider.get_partial_state() {
                Ok(new_state) => {
                    let mut current = self.state.write().await;
                    if *current != new_state {
                        info!(state = ?new_state, "Agent state changed");
                        *current = new_state.clone();
                        let _ = self.notifier.send(new_state);
                    }
                }
                Err(e) => warn!("Failed to poll agent status: {e}"),
            }
        }
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
                let wazuh_update = compute_component_update(
                    "Wazuh Agent",
                    &current_state.version,
                    &current_state.groups,
                    &info.wazuh,
                    &info.prerelease_test_groups,
                );

                let tray_update = compute_component_update(
                    "Tray App",
                    &current_state.tray_version,
                    &[], // No pre-release groups for tray app yet
                    &info.tray,
                    &[],
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
    online: &crate::models::FrameworkVersion,
    prerelease_groups: &[String],
) -> crate::models::ComponentUpdate {
    if local_version == "Unknown" || local_version == "Not Installed" {
        return crate::models::ComponentUpdate {
            name: name.to_string(),
            current_version: local_version.to_string(),
            latest_version: online.version.clone(),
            state: crate::models::UpdateState::Unknown,
            can_update: false,
        };
    }

    let is_outdated = !online.version.is_empty()
        && crate::version_utils::is_version_higher(&online.version, local_version);

    let has_prerelease = !online.prerelease_version.is_empty()
        && should_show_prerelease_for_component(prerelease_groups, agent_groups)
        && crate::version_utils::is_version_higher(&online.prerelease_version, local_version);

    let (state, latest, can_update) = if is_outdated {
        (crate::models::UpdateState::Outdated, online.version.clone(), true)
    } else if has_prerelease {
        (crate::models::UpdateState::PrereleaseAvailable, online.prerelease_version.clone(), true)
    } else {
        (crate::models::UpdateState::UpToDate, online.version.clone(), false)
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
