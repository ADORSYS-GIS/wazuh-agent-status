//! Agent state manager — owns the single source of truth for local agent state,
//! broadcasts changes to subscribers, and provides on-demand version checking.

use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time;
use tracing::{info, warn};

use crate::config::{AgentPaths, Config};
use crate::models::{AgentState, VersionInfo};
use crate::status_provider::StatusProvider;
use crate::updater::NativeUpdater;
use crate::version_utils::{fetch_version_info, is_version_higher, should_show_prerelease};

// ── Version cache ─────────────────────────────────────────────────────────────

struct VersionCache {
    /// The human-readable status string sent to clients.
    status: String,
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
    /// Update orchestrator.
    updater: NativeUpdater,
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
        paths: Arc<AgentPaths>,
        provider: Box<dyn StatusProvider>,
    ) -> Self {
        let (tx, _) = broadcast::channel(128);
        let updater = NativeUpdater::new(Arc::clone(&paths), config.version_url.clone());

        Self {
            state: Arc::new(RwLock::new(AgentState::default())),
            notifier: tx,
            provider,
            updater,
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
    pub async fn get_version_status(&self) -> String {
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
                let status = compute_version_status(&current_state.version, &current_state.groups, &info);
                let final_status = if is_fallback {
                    format!("{status} (cached)")
                } else {
                    status
                };

                let mut cache = self.version_cache.write().await;
                *cache = Some(VersionCache {
                    status: final_status.clone(),
                    info,
                    fetched_at: if is_fallback { 
                        // Keep old timestamp to force retry later
                        self.version_cache.read().await.as_ref().map(|c| c.fetched_at).unwrap_or(now)
                    } else { 
                        now 
                    },
                });
                final_status
            }
            None => {
                warn!("Failed to fetch remote version manifest and no cache available");
                "Version: Unknown (network error)".to_string()
            }
        }
    }

    // ── Update ────────────────────────────────────────────────────────────────

    /// Kick off an agent update and stream `UPDATE_PROGRESS: <msg>` strings
    /// through `tx`.
    pub async fn run_update(&self, prerelease: bool, tx: mpsc::Sender<String>) {
        self.updater.run_update(prerelease, tx).await;
    }
}

// ── Version status computation ────────────────────────────────────────────────

fn compute_version_status(
    local_version: &str,
    agent_groups: &[String],
    online: &VersionInfo,
) -> String {
    if local_version == "Unknown" || local_version == "Not Installed" {
        return format!("Version: {local_version}");
    }

    let is_current_prerelease = local_version.contains("rc");
    let version_prefix = if is_current_prerelease {
        format!("Prerelease: v{local_version}")
    } else {
        format!("v{local_version}")
    };

    let is_outdated = !online.framework.version.is_empty()
        && is_version_higher(&online.framework.version, local_version);

    let has_prerelease = !online.framework.prerelease_version.is_empty()
        && should_show_prerelease(online, agent_groups)
        && is_version_higher(&online.framework.prerelease_version, local_version);

    match (is_outdated, has_prerelease) {
        (true, true) => format!(
            "Outdated with Prerelease available: {} (stable: {}, prerelease: {})",
            version_prefix, online.framework.version, online.framework.prerelease_version
        ),
        (true, false) => format!("Outdated, {version_prefix}"),
        (false, true) => format!(
            "Prerelease available: {} (current: {})",
            online.framework.prerelease_version, version_prefix
        ),
        (false, false) => format!("Up to date, {version_prefix}"),
    }
}
