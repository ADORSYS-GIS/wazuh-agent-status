use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{self, Duration};
use tracing::{info, warn};

use crate::models::{AgentState, AgentStatus, ConnectionStatus, VersionInfo};
use crate::status_provider::{StatusProvider, NativeStatusProvider};
use crate::version_utils::{self, fetch_version_info, is_version_higher, should_show_prerelease};

const VERSION_URL: &str = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/heads/main/versions.json";

pub struct AgentManager {
    state: Arc<RwLock<AgentState>>,
    notifier: broadcast::Sender<AgentState>,
    provider: Box<dyn StatusProvider>,
}

impl AgentManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        let provider = Box::new(NativeStatusProvider::new());
        
        Self {
            state: Arc::new(RwLock::new(AgentState::default())),
            notifier: tx,
            provider,
        }
    }

    pub async fn get_state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentState> {
        self.notifier.subscribe()
    }

    pub async fn start_polling(&self, interval: Duration) {
        let mut ticker = time::interval(interval);
        
        loop {
            ticker.tick().await;
            
            match self.provider.get_partial_state() {
                Ok(mut new_state) => {
                    // Fetch online version info
                    if let Some(online_info) = fetch_version_info(VERSION_URL).await {
                        new_state.online_version_status = compute_version_status(&new_state, &online_info);
                    }

                    let mut current_state = self.state.write().await;
                    if *current_state != new_state {
                        info!("Agent state changed: {:?}", new_state);
                        *current_state = new_state.clone();
                        // Notify subscribers
                        let _ = self.notifier.send(new_state);
                    }
                }
                Err(e) => {
                    warn!("Failed to poll agent status: {:?}", e);
                }
            }
        }
    }
}

fn compute_version_status(state: &AgentState, online: &VersionInfo) -> String {
    let local_version = &state.version;
    if local_version == "Unknown" {
        return "Version: Unknown".to_string();
    }

    let is_current_prerelease = local_version.contains("rc");
    let version_prefix = if is_current_prerelease {
        format!("Prerelease: v{}", local_version)
    } else {
        format!("v{}", local_version)
    };

    let is_outdated = !online.framework.version.is_empty() && is_version_higher(&online.framework.version, local_version);
    let has_prerelease = !online.framework.prerelease_version.is_empty() 
        && should_show_prerelease(online, &state.groups) 
        && is_version_higher(&online.framework.prerelease_version, local_version);

    if is_outdated && has_prerelease {
        format!("Outdated with Prerelease available: {} (stable: {}, prerelease: {})", 
            version_prefix, online.framework.version, online.framework.prerelease_version)
    } else if is_outdated {
        format!("Outdated, {}", version_prefix)
    } else if has_prerelease {
        format!("Prerelease available: {} (current: {})", 
            online.framework.prerelease_version, version_prefix)
    } else {
        format!("Up to date, {}", version_prefix)
    }
}
