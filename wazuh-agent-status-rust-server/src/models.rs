use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Active,
    Inactive,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionInfo {
    pub framework: FrameworkVersion,
    pub prerelease_test_groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameworkVersion {
    pub version: String,
    pub prerelease_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentState {
    pub status: AgentStatus,
    pub connection: ConnectionStatus,
    pub version: String,
    pub groups: Vec<String>,
    pub online_version_status: String,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status: AgentStatus::Unknown,
            connection: ConnectionStatus::Unknown,
            version: "Unknown".to_string(),
            groups: Vec::new(),
            online_version_status: "Unknown".to_string(),
        }
    }
}
