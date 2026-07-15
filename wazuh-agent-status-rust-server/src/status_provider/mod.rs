//! `StatusProvider` trait and platform-specific provider registration.

use std::fs;
use std::path::Path;

use crate::config::AgentPaths;
use crate::errors::{Result, ServerError};
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

/// Read connection status from the Wazuh state file without scanning the
/// process list.  Used by [`get_partial_state()`] after a prior metrics scan.
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

/// Abstraction over platform-specific Wazuh agent status retrieval.
///
/// The default implementation of [`get_partial_state()`] composes the
/// individual methods to build a complete [`AgentState`] with a single
/// process scan.
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

    /// Optional: return the path to the Wazuh state file so that
    /// [`get_partial_state()`] can determine the connection status
    /// without a second process scan.
    fn get_state_file_path(&self) -> Option<&Path> {
        None
    }

    /// Compose a full [`AgentState`] from the individual methods.
    ///
    /// This implementation performs **one** process scan (via
    /// [`get_system_metrics()`]) and reuses the result to derive
    /// both the agent status and (when [`get_state_file_path()`] is
    /// provided) the connection status, eliminating a second scan.
    ///
    /// Note: `online_version_status` is intentionally excluded — it is an
    /// on-demand operation handled by [`crate::manager::AgentManager`].
    fn get_partial_state(&self) -> Result<AgentState> {
        let metrics = self.get_system_metrics()?;

        // Derive status from wazuh-agentd presence (consistent with connection check).
        let status = if metrics.agentd_found {
            AgentStatus::Active
        } else {
            AgentStatus::Inactive
        };

        // Reuse the metrics scan to decide connection without a second scan.
        let connection = if let Some(path) = self.get_state_file_path() {
            if !metrics.agentd_found {
                ConnectionStatus::Disconnected
            } else {
                read_connection_from_state_file(path)?
            }
        } else {
            // Fallback for providers that don't expose their state file path.
            self.get_connection_status()?
        };

        Ok(AgentState {
            status,
            connection,
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
