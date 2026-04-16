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
    pub fn new(config: Arc<Config>, paths: Arc<AgentPaths>) -> Self {
        let (tx, _) = broadcast::channel(128);
        let provider = Box::new(crate::status_provider::native_provider(
            paths.as_ref().clone(),
        ));
        let updater = NativeUpdater::new(Arc::clone(&paths), config.version_url.clone());

        Self {
            state:         Arc::new(RwLock::new(AgentState::default())),
            notifier:      tx,
            provider,
            updater,
            version_cache: RwLock::new(None),
            config,
        }
    }

    // ── State access ──────────────────────────────────────────────────────────

    /// Return a snapshot of the current local agent state.
    pub async fn get_state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    /// Subscribe to state-change notifications.
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
        // Fast path — return cached value if still fresh.
        {
            let cache = self.version_cache.read().await;
            if let Some(c) = &*cache {
                if c.fetched_at.elapsed() < self.config.version_cache_ttl {
                    return c.status.clone();
                }
            }
        }

        // Slow path — fetch fresh data.
        let current_state = self.get_state().await;
        let status = match fetch_version_info(&self.config.version_url).await {
            Some(info) => compute_version_status(&current_state.version, &current_state.groups, &info),
            None => {
                warn!("Failed to fetch remote version manifest");
                "Version: Unknown (network error)".to_string()
            }
        };

        let mut cache = self.version_cache.write().await;
        *cache = Some(VersionCache {
            status: status.clone(),
            fetched_at: Instant::now(),
        });

        status
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
    if local_version == "Unknown" {
        return "Version: Unknown".to_string();
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
