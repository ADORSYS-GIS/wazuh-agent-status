use std::process::Command;
use crate::models::{AgentStatus, ConnectionStatus};
use crate::status_provider::StatusProvider;
use crate::errors::{Result, ServerError};
use crate::group_extractor;

pub struct MacosStatusProvider;

impl MacosStatusProvider {
    pub fn new() -> Self {
        Self
    }

    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<String> {
        let output = Command::new("sudo")
            .arg(cmd)
            .args(args)
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(ServerError::PlatformError(err));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl StatusProvider for MacosStatusProvider {
    fn get_agent_status(&self) -> Result<AgentStatus> {
        let output = self.run_command("/Library/Ossec/bin/wazuh-control", &["status"])?;
        if output.contains("wazuh-agentd is running") {
            Ok(AgentStatus::Active)
        } else {
            Ok(AgentStatus::Inactive)
        }
    }

    fn get_connection_status(&self) -> Result<ConnectionStatus> {
        let output = self.run_command("grep", &["^status", "/Library/Ossec/var/run/wazuh-agentd.state"])?;
        if output.contains("status='connected'") {
            Ok(ConnectionStatus::Connected)
        } else {
            Ok(ConnectionStatus::Disconnected)
        }
    }

    fn get_agent_groups(&self) -> Result<Vec<String>> {
        group_extractor::extract_groups("/Library/Ossec/etc/shared/merged.mg")
            .map_err(|e| ServerError::PlatformError(e.to_string()))
    }

    fn get_agent_version(&self) -> Result<String> {
        let output = self.run_command("cat", &["/Library/Ossec/etc/version.txt"])?;
        Ok(output.trim().to_string())
    }
}
