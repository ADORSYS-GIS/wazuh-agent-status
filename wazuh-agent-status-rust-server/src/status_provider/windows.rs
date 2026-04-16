use std::process::Command;
use crate::models::{AgentStatus, ConnectionStatus};
use crate::status_provider::StatusProvider;
use crate::errors::{Result, ServerError};
use crate::group_extractor;

pub struct WindowsStatusProvider;

impl WindowsStatusProvider {
    pub fn new() -> Self {
        Self
    }

    fn run_powershell(&self, command: &str) -> Result<String> {
        let output = Command::new("powershell.exe")
            .args(&["-NoProfile", "-NonInteractive", "-Command", command])
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(ServerError::PlatformError(err));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl StatusProvider for WindowsStatusProvider {
    fn get_agent_status(&self) -> Result<AgentStatus> {
        let output = self.run_powershell("Get-Service -Name WazuhSvc")?;
        if output.contains("Running") {
            Ok(AgentStatus::Active)
        } else {
            Ok(AgentStatus::Inactive)
        }
    }

    fn get_connection_status(&self) -> Result<ConnectionStatus> {
        let output = self.run_powershell("Select-String -Path 'C:\\Program Files (x86)\\ossec-agent\\wazuh-agent.state' -Pattern '^status'")?;
        if output.contains("status='connected'") {
            Ok(ConnectionStatus::Connected)
        } else {
            Ok(ConnectionStatus::Disconnected)
        }
    }

    fn get_agent_groups(&self) -> Result<Vec<String>> {
        group_extractor::extract_groups("C:\\Program Files (x86)\\ossec-agent\\shared\\merged.mg")
            .map_err(|e| ServerError::PlatformError(e.to_string()))
    }

    fn get_agent_version(&self) -> Result<String> {
        let output = self.run_powershell("Get-Content 'C:\\Program Files (x86)\\ossec-agent\\version.txt' -ErrorAction SilentlyContinue")?;
        Ok(output.trim().to_string())
    }
}
