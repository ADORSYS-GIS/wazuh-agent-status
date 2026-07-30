use std::fs;
use std::path::Path;

use crate::config::AgentPaths;
use crate::errors::{Result, ServerError};
use crate::models::{AgentState, AgentStatus, ConnectionStatus, SystemMetrics};

#[cfg(target_os = "linux")]
const LOGCOLLECTOR_NAME: &str = "wazuh-logcollec";
#[cfg(not(target_os = "linux"))]
const LOGCOLLECTOR_NAME: &str = "wazuh-logcollector";

pub const UNIX_AGENT_PROCESSES: &[&str] = &[
    "wazuh-agentd",
    "wazuh-modulesd",
    LOGCOLLECTOR_NAME,
    "wazuh-syscheckd",
    "wazuh-execd",
];

pub const WINDOWS_AGENT_PROCESSES: &[&str] = &[
    "ossec-agent.exe",
    "ossec-agentd.exe",
    "ossec-logcollector.exe",
    "ossec-syscheckd.exe",
    "ossec-execd.exe",
    "wazuh-agent.exe",
    "wazuh-agentd.exe",
    "wazuh-logcollector.exe",
    "wazuh-syscheckd.exe",
    "wazuh-execd.exe",
];

pub(crate) fn read_connection_from_state_file(path: &Path) -> Result<ConnectionStatus> {
    match fs::read_to_string(path) {
        Ok(content) if content.contains("status='connected'") => Ok(ConnectionStatus::Connected),
        Ok(_) => Ok(ConnectionStatus::Disconnected),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ConnectionStatus::Disconnected),
        Err(e) => Err(ServerError::PlatformError(format!(
            "Cannot read state file {}: {e}",
            path.display()
        ))),
    }
}

pub trait StatusProvider: Send + Sync {
    fn get_agent_status(&self) -> Result<AgentStatus>;
    fn get_connection_status(&self) -> Result<ConnectionStatus>;
    fn get_agent_version(&self) -> Result<String>;
    fn get_tray_version(&self) -> Result<String>;
    fn get_agent_groups(&self) -> Result<Vec<String>>;
    fn get_system_metrics(&self) -> Result<SystemMetrics>;

    fn get_state_file_path(&self) -> Option<&Path> {
        None
    }

    fn get_partial_state(&self) -> Result<AgentState> {
        let metrics = self.get_system_metrics()?;

        let status = if metrics.agentd_found {
            AgentStatus::Active
        } else {
            AgentStatus::Inactive
        };

        let connection = if let Some(path) = self.get_state_file_path() {
            if !metrics.agentd_found {
                ConnectionStatus::Disconnected
            } else {
                read_connection_from_state_file(path)?
            }
        } else {
            self.get_connection_status()?
        };

        Ok(AgentState {
            status,
            connection,
            version: self.get_agent_version()?,
            tray_version: self.get_tray_version()?,
            groups: self.get_agent_groups()?,
            metrics,
            self_healing_enabled: true,
            agent_id: String::new(),
            agent_name: String::new(),
            agent_key: String::new(),
        })
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub mod unix;

#[cfg(target_os = "windows")]
pub(crate) const WINDOWS_EXE_PREFIXES: &[&str] = &[
    "c:\\program files\\wazuh agent\\",
    "c:\\program files (x86)\\wazuh agent\\",
    "c:\\program files\\ossec-agent\\",
    "c:\\program files (x86)\\ossec-agent\\",
];

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use linux::LinuxStatusProvider as NativeStatusProvider;
#[cfg(target_os = "macos")]
pub use macos::MacosStatusProvider as NativeStatusProvider;
#[cfg(target_os = "windows")]
pub use windows::WindowsStatusProvider as NativeStatusProvider;

pub fn native_provider(paths: AgentPaths) -> NativeStatusProvider {
    NativeStatusProvider::new(paths)
}
