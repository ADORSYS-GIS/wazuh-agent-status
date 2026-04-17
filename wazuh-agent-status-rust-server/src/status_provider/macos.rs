//! macOS status provider — reads Wazuh agent state directly from the file
//! system without requiring `sudo`.
//!
//! # Permission model
//!
//! Files under `/Library/Ossec` are owned by the `ossec` group.  Add the
//! service user to that group once at deployment:
//!
//! ```bash
//! sudo dseditgroup -o edit -a <service-user> -t user ossec
//! ```

use std::fs;
use std::process::Command;

use crate::config::AgentPaths;
use crate::errors::{Result, ServerError};
use crate::group_extractor;
use crate::models::{AgentStatus, ConnectionStatus};
use crate::status_provider::StatusProvider;

pub struct MacosStatusProvider {
    paths: AgentPaths,
}

impl MacosStatusProvider {
    pub fn new(paths: AgentPaths) -> Self {
        Self { paths }
    }

    /// Determine whether `wazuh-agentd` is alive by running `wazuh-control status`.
    fn is_agent_running(&self) -> bool {
        let control_path = self.paths.state_file.parent() // var/run/
            .and_then(|p| p.parent())                    // var/
            .and_then(|p| p.parent())                    // base/
            .map(|base| base.join("bin/wazuh-control"))
            .unwrap_or_else(|| std::path::PathBuf::from("/Library/Ossec/bin/wazuh-control"));

        if let Ok(output) = Command::new(control_path)
            .arg("status")
            .output() 
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("wazuh-agentd is running") {
                return true;
            }
        }
        
        // Fallback: Check if the PID file exists and the process in it is alive
        self.read_pid()
            .map(|pid| {
                Command::new("ps")
                    .args(["-p", &pid.to_string()])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    fn read_pid(&self) -> Option<u32> {
        fs::read_to_string(&self.paths.pid_file)
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }
}

impl StatusProvider for MacosStatusProvider {
    fn get_agent_status(&self) -> Result<AgentStatus> {
        if self.is_agent_running() {
            Ok(AgentStatus::Active)
        } else {
            Ok(AgentStatus::Inactive)
        }
    }

    fn get_connection_status(&self) -> Result<ConnectionStatus> {
        let content = match fs::read_to_string(&self.paths.state_file) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Agent stopped — correctly reflect as Disconnected
                return Ok(ConnectionStatus::Disconnected);
            }
            Err(e) => {
                return Err(ServerError::PlatformError(format!(
                    "Cannot read state file {}: {e}",
                    self.paths.state_file.display()
                )));
            }
        };

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
