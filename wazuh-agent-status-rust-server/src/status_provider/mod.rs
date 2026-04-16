use crate::models::{AgentStatus, ConnectionStatus, AgentState};
use crate::errors::Result;

pub trait StatusProvider: Send + Sync {
    fn get_agent_status(&self) -> Result<AgentStatus>;
    fn get_connection_status(&self) -> Result<ConnectionStatus>;
    fn get_agent_version(&self) -> Result<String>;
    fn get_agent_groups(&self) -> Result<Vec<String>>;

    fn get_partial_state(&self) -> Result<AgentState> {
        Ok(AgentState {
            status: self.get_agent_status()?,
            connection: self.get_connection_status()?,
            version: self.get_agent_version()?,
            groups: self.get_agent_groups()?,
            online_version_status: "Unknown".to_string(),
        })
    }
}

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub use linux::LinuxStatusProvider as NativeStatusProvider;
#[cfg(target_os = "windows")]
pub use windows::WindowsStatusProvider as NativeStatusProvider;
#[cfg(target_os = "macos")]
pub use macos::MacosStatusProvider as NativeStatusProvider;
