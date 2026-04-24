//! Core data models shared across the server.

use serde::{Deserialize, Serialize};

/// Running state of the local Wazuh agent daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Inactive,
    Unknown,
}

/// Network connection state of the local Wazuh agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Mapping of component versions
    pub components: std::collections::HashMap<String, ComponentVersion>,
    /// Global framework versioning
    pub framework: FrameworkVersion,
    #[serde(alias = "prerelease_test_groups", default)]
    pub prerelease_test_groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVersion {
    pub version: String,
    #[serde(default)]
    pub prerelease_version: String,
}

/// Version numbers within the online manifest for the framework (tray app).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkVersion {
    pub version: String,
    #[serde(default)]
    pub prerelease_version: String,
}

/// Real-time system performance indicators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage:    0.0,
            memory_usage: 0.0,
            total_memory: 0,
            used_memory:  0,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UpdateState {
    UpToDate,
    Outdated,
    PrereleaseAvailable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentUpdate {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub state: UpdateState,
    pub can_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub wazuh: ComponentUpdate,
    pub tray: ComponentUpdate,
    pub has_updates: bool,
}

/// Complete local state of the Wazuh agent, polled on each tick.
///
/// This is what gets broadcast to subscribers on every change.
/// The online version status is intentionally excluded here — it is
/// fetched on-demand via [`crate::manager::AgentManager::get_version_status`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentState {
    pub status: AgentStatus,
    pub connection: ConnectionStatus,
    /// Locally installed agent version string (e.g. `"4.7.2"`).
    pub version: String,
    /// Tray application version string.
    pub tray_version: String,
    /// Agent group memberships parsed from `merged.mg`.
    pub groups: Vec<String>,
    /// System performance indicators.
    pub metrics: SystemMetrics,
    /// Whether self-healing is currently active on the server.
    pub self_healing_enabled: bool,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status:               AgentStatus::Unknown,
            connection:           ConnectionStatus::Unknown,
            version:              "Unknown".to_string(),
            tray_version:         "Unknown".to_string(),
            groups:               Vec::new(),
            metrics:              SystemMetrics::default(),
            self_healing_enabled: true,
        }
    }
}
