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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    pub framework: FrameworkVersion,
    #[serde(alias = "prerelease_test_grouops", default)]
    pub prerelease_test_groups: Vec<String>,
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

/// Version numbers within the online manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkVersion {
    pub version: String,
    pub prerelease_version: String,
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
    /// Agent group memberships parsed from `merged.mg`.
    pub groups: Vec<String>,
    /// System performance indicators.
    pub metrics: SystemMetrics,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status:     AgentStatus::Unknown,
            connection: ConnectionStatus::Unknown,
            version:    "Unknown".to_string(),
            groups:     Vec::new(),
            metrics:    SystemMetrics::default(),
        }
    }
}
