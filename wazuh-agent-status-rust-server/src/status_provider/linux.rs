//! Linux status provider — reads Wazuh agent state directly from the file
//! system without requiring `sudo`.

use std::fs;
use sysinfo::System;

use crate::config::AgentPaths;
use crate::errors::{Result, ServerError};
use crate::group_extractor;
use crate::models::{AgentStatus, ConnectionStatus};
use crate::status_provider::StatusProvider;

pub struct LinuxStatusProvider {
    paths: AgentPaths,
    sys: std::sync::Mutex<System>,
}

impl LinuxStatusProvider {
    pub fn new(paths: AgentPaths) -> Self {
        let mut sys = System::new();
        sys.refresh_all();
        Self {
            paths,
            sys: std::sync::Mutex::new(sys),
        }
    }

    fn is_agent_running(&self) -> bool {
        if let Ok(mut sys) = self.sys.lock() {
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            let is_running = sys
                .processes()
                .values()
                .any(|p| p.name().to_string_lossy() == "wazuh-agentd");
            if !is_running {
                tracing::info!("Process list check confirms wazuh-agentd is NOT running");
            }
            is_running
        } else {
            false
        }
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
        // Optimization: If the agent service is stopped, it's definitely disconnected
        // regardless of what the stale state file says.
        if !self.is_agent_running() {
            return Ok(ConnectionStatus::Disconnected);
        }

        let content = match fs::read_to_string(&self.paths.state_file) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
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
        match fs::read_to_string(&self.paths.version_json) {
            Ok(content) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
                    && let Some(v) = json.get("version").and_then(|v| v.as_str())
                {
                    let version = v.to_string();
                    tracing::debug!(version = %version, path = %self.paths.version_json.display(), "Read agent version from VERSION.json");
                    return Ok(version);
                }
                tracing::warn!(path = %self.paths.version_json.display(), "Failed to parse version from VERSION.json");
                Ok("Unknown".to_string())
            }
            Err(e) => {
                tracing::warn!(error = %e, path = %self.paths.version_json.display(), "Failed to read VERSION.json");
                Ok("Unknown".to_string())
            }
        }
    }

    fn get_tray_version(&self) -> Result<String> {
        match fs::read_to_string(&self.paths.version_file) {
            Ok(raw) => {
                let v = raw.trim().to_string();
                tracing::debug!(version = %v, path = %self.paths.version_file.display(), "Read Wazuh Agent Setup Version");
                Ok(v)
            }
            Err(e) => {
                tracing::warn!(error = %e, path = %self.paths.version_file.display(), "Failed to read tray version");
                Ok("Unknown".to_string())
            }
        }
    }

    fn get_agent_groups(&self) -> Result<Vec<String>> {
        match group_extractor::extract_groups(&self.paths.merged_mg) {
            Ok(groups) => Ok(groups),
            Err(_) => Ok(Vec::new()), // No groups if agent is not installed
        }
    }

    fn get_system_metrics(&self) -> Result<crate::models::SystemMetrics> {
        let mut sys = self
            .sys
            .lock()
            .map_err(|_| ServerError::PlatformError("Failed to lock system metrics".to_string()))?;

        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sys.refresh_memory();
        sys.refresh_cpu_all();

        Ok(crate::status_provider::unix::collect_unix_system_metrics(
            &sys,
        ))
    }
}
