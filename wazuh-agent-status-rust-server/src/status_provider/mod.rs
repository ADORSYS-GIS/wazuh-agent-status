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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    struct DummyProvider {
        metrics: SystemMetrics,
        has_state_path: bool,
        state_file: Option<std::path::PathBuf>,
    }

    impl StatusProvider for DummyProvider {
        fn get_agent_status(&self) -> Result<AgentStatus> {
            Ok(AgentStatus::Active)
        }
        fn get_connection_status(&self) -> Result<ConnectionStatus> {
            Ok(ConnectionStatus::Connected)
        }
        fn get_agent_version(&self) -> Result<String> {
            Ok("4.8.0".to_string())
        }
        fn get_tray_version(&self) -> Result<String> {
            Ok("1.2.0".to_string())
        }
        fn get_agent_groups(&self) -> Result<Vec<String>> {
            Ok(vec!["default".to_string()])
        }
        fn get_system_metrics(&self) -> Result<SystemMetrics> {
            Ok(self.metrics.clone())
        }
        fn get_state_file_path(&self) -> Option<&Path> {
            if self.has_state_path {
                self.state_file.as_deref()
            } else {
                None
            }
        }
    }

    #[test]
    fn test_read_connection_from_state_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "# State file\nstatus='connected'\nlast_keepalive='2026-10-21'"
        )
        .unwrap();
        assert_eq!(
            read_connection_from_state_file(file.path()).unwrap(),
            ConnectionStatus::Connected
        );

        let mut disc_file = NamedTempFile::new().unwrap();
        writeln!(disc_file, "status='disconnected'").unwrap();
        assert_eq!(
            read_connection_from_state_file(disc_file.path()).unwrap(),
            ConnectionStatus::Disconnected
        );

        let non_existent = std::path::PathBuf::from("/path/that/does/not/exist/ever");
        assert_eq!(
            read_connection_from_state_file(&non_existent).unwrap(),
            ConnectionStatus::Disconnected
        );

        let temp_dir = tempfile::tempdir().unwrap();
        // Reading a directory as file triggers an I/O error
        let err = read_connection_from_state_file(temp_dir.path());
        assert!(err.is_err());
    }

    #[test]
    fn test_get_partial_state_with_state_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "status='connected'").unwrap();

        let provider = DummyProvider {
            metrics: SystemMetrics {
                agentd_found: true,
                agent_found: true,
                cpu_usage: 1.0,
                memory_usage: 2.0,
                total_memory: 1000,
                used_memory: 200,
            },
            has_state_path: true,
            state_file: Some(file.path().to_path_buf()),
        };

        let state = provider.get_partial_state().unwrap();
        assert_eq!(state.status, AgentStatus::Active);
        assert_eq!(state.connection, ConnectionStatus::Connected);
        assert_eq!(state.version, "4.8.0");
        assert_eq!(state.tray_version, "1.2.0");
        assert_eq!(state.groups, vec!["default".to_string()]);

        // Agentd not found => status Inactive and connection Disconnected
        let inactive_provider = DummyProvider {
            metrics: SystemMetrics {
                agentd_found: false,
                agent_found: false,
                cpu_usage: 0.0,
                memory_usage: 0.0,
                total_memory: 1000,
                used_memory: 0,
            },
            has_state_path: true,
            state_file: Some(file.path().to_path_buf()),
        };
        let inactive_state = inactive_provider.get_partial_state().unwrap();
        assert_eq!(inactive_state.status, AgentStatus::Inactive);
        assert_eq!(inactive_state.connection, ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_get_partial_state_without_state_file() {
        let provider = DummyProvider {
            metrics: SystemMetrics {
                agentd_found: true,
                agent_found: true,
                cpu_usage: 1.0,
                memory_usage: 2.0,
                total_memory: 1000,
                used_memory: 200,
            },
            has_state_path: false,
            state_file: None,
        };

        let state = provider.get_partial_state().unwrap();
        assert_eq!(state.status, AgentStatus::Active);
        assert_eq!(state.connection, ConnectionStatus::Connected);
    }
}
