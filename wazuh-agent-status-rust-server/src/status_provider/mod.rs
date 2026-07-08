//! `StatusProvider` trait and platform-specific provider registration.

use crate::config::AgentPaths;
use crate::errors::Result;
use crate::models::{AgentState, AgentStatus, ConnectionStatus, SystemMetrics};

// Linux /proc/[pid]/comm truncates names to 15 characters
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
    // Wazuh 4.x on Windows (original naming)
    "ossec-agent.exe",
    "ossec-agentd.exe",
    "ossec-logcollector.exe",
    "ossec-syscheckd.exe",
    "ossec-execd.exe",
    // Wazuh 5.x+ on Windows (renamed binaries)
    "wazuh-agent.exe",
    "wazuh-agentd.exe",
    "wazuh-logcollector.exe",
    "wazuh-syscheckd.exe",
    "wazuh-execd.exe",
];

/// Abstraction over platform-specific Wazuh agent status retrieval.
///
/// The default implementation of [`get_partial_state`] composes the individual
/// methods to build a complete [`AgentState`].  Implementors only need to
/// provide the four leaf methods.
pub trait StatusProvider: Send + Sync {
    fn get_agent_status(&self) -> Result<AgentStatus>;
    fn get_connection_status(&self) -> Result<ConnectionStatus>;
    fn get_agent_version(&self) -> Result<String>;
    /// Get the version of the tray application.
    fn get_tray_version(&self) -> Result<String>;
    /// Get groups assigned to this agent.
    fn get_agent_groups(&self) -> Result<Vec<String>>;
    /// Get system-wide performance metrics.
    fn get_system_metrics(&self) -> Result<SystemMetrics>;

    /// Compose a full [`AgentState`] from the individual methods.
    ///
    /// Note: `online_version_status` is intentionally excluded — it is an
    /// on-demand operation handled by [`crate::manager::AgentManager`].
    fn get_partial_state(&self) -> Result<AgentState> {
        // Call get_system_metrics first — it does a full process scan and sets
        // agent_found. We derive agent status from that instead of calling
        // get_agent_status() separately, which would re-scan the process list.
        let metrics = self.get_system_metrics()?;

        let status = if metrics.agent_found {
            AgentStatus::Active
        } else {
            AgentStatus::Inactive
        };

        Ok(AgentState {
            status,
            connection: self.get_connection_status()?,
            version: self.get_agent_version()?,
            tray_version: self.get_tray_version()?,
            groups: self.get_agent_groups()?,
            metrics,
            self_healing_enabled: true, // Initial placeholder; overridden by Manager config
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
