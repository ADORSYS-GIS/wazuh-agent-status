//! Windows status provider.
//!
//! Agent status is queried via PowerShell `Get-Service` because Windows has no
//! equivalent of a Unix PID file that can be checked without elevated rights.
//! All other data (connection state, version, groups) is read directly from
//! the file system — no elevated privileges required for those reads.

use std::fs;
use std::process::Command;

use crate::config::AgentPaths;
use crate::errors::{Result, ServerError};
use crate::group_extractor;
use crate::models::{AgentStatus, ConnectionStatus};
use crate::status_provider::StatusProvider;

pub struct WindowsStatusProvider {
    paths: AgentPaths,
}

impl WindowsStatusProvider {
    pub fn new(paths: AgentPaths) -> Self {
        Self { paths }
    }

    /// Run a PowerShell command and return trimmed stdout.
    fn run_powershell(&self, command: &str) -> Result<String> {
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", command])
            .output()?;

        if !output.status.success() {
            let msg = String::from_utf8_lossy(&output.stderr).into_owned();
            return Err(ServerError::PlatformError(msg));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl StatusProvider for WindowsStatusProvider {
    fn get_agent_status(&self) -> Result<AgentStatus> {
        // PowerShell is the only practical way to query service state on Windows.
        let output = self.run_powershell("(Get-Service -Name WazuhSvc -ErrorAction SilentlyContinue).Status")?;
        if output.eq_ignore_ascii_case("running") {
            Ok(AgentStatus::Active)
        } else {
            Ok(AgentStatus::Inactive)
        }
    }

    fn get_connection_status(&self) -> Result<ConnectionStatus> {
        // Direct file read — no PowerShell needed.
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
        // Direct file read — no PowerShell needed.
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
