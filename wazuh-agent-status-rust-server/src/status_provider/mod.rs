//! `StatusProvider` trait and platform-specific provider registration.

use crate::config::AgentPaths;
use crate::errors::Result;
use crate::models::{AgentState, AgentStatus, ConnectionStatus};

/// Abstraction over platform-specific Wazuh agent status retrieval.
///
/// The default implementation of [`get_partial_state`] composes the individual
/// methods to build a complete [`AgentState`].  Implementors only need to
/// provide the four leaf methods.
pub trait StatusProvider: Send + Sync {
    fn get_agent_status(&self) -> Result<AgentStatus>;
    fn get_connection_status(&self) -> Result<ConnectionStatus>;
    fn get_agent_version(&self) -> Result<String>;
    fn get_agent_groups(&self) -> Result<Vec<String>>;
    fn get_system_metrics(&self) -> Result<crate::models::SystemMetrics>;

    /// Compose a full [`AgentState`] from the individual methods.
    ///
    /// Note: `online_version_status` is intentionally excluded — it is an
    /// on-demand operation handled by [`crate::manager::AgentManager`].
    fn get_partial_state(&self) -> Result<AgentState> {
        Ok(AgentState {
            status:     self.get_agent_status()?,
            connection: self.get_connection_status()?,
            version:    self.get_agent_version()?,
            groups:     self.get_agent_groups()?,
            metrics:    self.get_system_metrics()?,
        })
    }
}

// ── Platform module declarations ──────────────────────────────────────────────

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

// ── NativeStatusProvider alias ────────────────────────────────────────────────

#[cfg(target_os = "linux")]
pub use linux::LinuxStatusProvider as NativeStatusProvider;

#[cfg(target_os = "macos")]
pub use macos::MacosStatusProvider as NativeStatusProvider;

#[cfg(target_os = "windows")]
pub use windows::WindowsStatusProvider as NativeStatusProvider;

/// Convenience constructor that wires the native provider to the given paths.
pub fn native_provider(paths: AgentPaths) -> NativeStatusProvider {
    NativeStatusProvider::new(paths)
}
