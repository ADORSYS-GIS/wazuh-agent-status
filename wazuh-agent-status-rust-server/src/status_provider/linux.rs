//! Linux status provider — reads Wazuh agent state directly from the file
//! system without requiring `sudo`.
//!
//! # Permission model
//!
//! All files under `/var/ossec` that this provider reads are owned by the
//! `ossec` group with group-read (`g+r`) permissions.  Add the user that runs
//! this server to the `ossec` group once at deployment time:
//!
//! ```bash
//! sudo usermod -aG ossec <service-user>
//! ```
//!
//! After that, no `sudo` is needed at runtime.

use std::fs;
use std::path::Path;

use crate::config::AgentPaths;
use crate::errors::{Result, ServerError};
use crate::group_extractor;
use crate::models::{AgentStatus, ConnectionStatus};
use crate::status_provider::StatusProvider;

pub struct LinuxStatusProvider {
    paths: AgentPaths,
}

impl LinuxStatusProvider {
    pub fn new(paths: AgentPaths) -> Self {
        Self { paths }
    }

    /// Determine whether the `wazuh-agentd` process is alive by:
    /// 1. Reading the PID from the daemon's PID file.
    /// 2. Checking whether `/proc/<pid>` exists (Linux kernel guarantee).
    fn is_agent_running(&self) -> bool {
        self.read_pid()
            .map(|pid| Path::new(&format!("/proc/{pid}")).exists())
            .unwrap_or(false)
    }

    /// Parse the numeric PID from the daemon's PID file.
    fn read_pid(&self) -> Option<u32> {
        fs::read_to_string(&self.paths.pid_file)
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }
}

impl StatusProvider for LinuxStatusProvider {
    fn get_agent_status(&self) -> Result<AgentStatus> {
        if self.is_agent_running() {
            Ok(AgentStatus::Active)
        } else {
            Ok(AgentStatus::Inactive)
        }
    }

    fn get_connection_status(&self) -> Result<ConnectionStatus> {
        let content = fs::read_to_string(&self.paths.state_file).map_err(|e| {
            ServerError::PlatformError(format!(
                "Cannot read state file {}: {e}",
                self.paths.state_file.display()
            ))
        })?;

        if content.contains("status='connected'") {
            Ok(ConnectionStatus::Connected)
        } else {
            Ok(ConnectionStatus::Disconnected)
        }
    }

    fn get_agent_version(&self) -> Result<String> {
        let raw = fs::read_to_string(&self.paths.version_file).map_err(|e| {
            ServerError::PlatformError(format!(
                "Cannot read version file {}: {e}",
                self.paths.version_file.display()
            ))
        })?;
        Ok(raw.trim().to_string())
    }

    fn get_agent_groups(&self) -> Result<Vec<String>> {
        group_extractor::extract_groups(&self.paths.merged_mg)
            .map_err(|e| ServerError::PlatformError(e.to_string()))
    }
}
