use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Inactive,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub framework: FrameworkVersion,
    #[serde(alias = "prerelease_test_groups", default)]
    pub prerelease_test_groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkVersion {
    pub version: String,
    #[serde(default)]
    pub prerelease_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
    #[serde(default)]
    pub agent_found: bool,
    #[serde(default)]
    pub agentd_found: bool,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            total_memory: 0,
            used_memory: 0,
            agent_found: false,
            agentd_found: false,
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
    pub tray: ComponentUpdate,
    pub has_updates: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentState {
    pub status: AgentStatus,
    pub connection: ConnectionStatus,
    pub version: String,
    pub tray_version: String,
    pub groups: Vec<String>,
    pub metrics: SystemMetrics,
    pub self_healing_enabled: bool,
    pub agent_id: String,
    pub agent_name: String,
    pub agent_key: String,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status: AgentStatus::Unknown,
            connection: ConnectionStatus::Unknown,
            version: "Unknown".to_string(),
            tray_version: "Unknown".to_string(),
            groups: Vec::new(),
            metrics: SystemMetrics::default(),
            self_healing_enabled: true,
            agent_id: String::new(),
            agent_name: String::new(),
            agent_key: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub raw: String,
    pub level: LogLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Unknown,
}

impl LogLine {
    pub fn from_raw(raw: String) -> Self {
        let upper = raw.to_uppercase();
        let level =
            if upper.contains("ERROR") || upper.contains("CRITICAL") || upper.contains("FATAL") {
                LogLevel::Error
            } else if upper.contains("WARNING") || upper.contains("WARN") {
                LogLevel::Warning
            } else if upper.contains("DEBUG") {
                LogLevel::Debug
            } else if upper.contains("INFO") {
                LogLevel::Info
            } else {
                LogLevel::Unknown
            };
        Self { raw, level }
    }
}
