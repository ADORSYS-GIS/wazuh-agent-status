use std::fs;
use std::path::Path;
use std::process::Command;
use sysinfo::System;

use crate::config::AgentPaths;
use crate::errors::{Result, ServerError};
use crate::group_extractor;
use crate::models::{AgentStatus, ConnectionStatus};
use crate::status_provider::{StatusProvider, read_connection_from_state_file};
use tracing::debug;

pub struct WindowsStatusProvider {
    paths: AgentPaths,
    sys: std::sync::Mutex<System>,
}

impl WindowsStatusProvider {
    pub fn new(paths: AgentPaths) -> Self {
        let mut sys = System::new();
        sys.refresh_all();
        Self {
            paths,
            sys: std::sync::Mutex::new(sys),
        }
    }

    fn run_powershell(&self, command: &str) -> Result<String> {
        let output = Command::new("powershell.exe")
            .args([
                "-ExecutionPolicy",
                "Bypass",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                command,
            ])
            .output()?;

        if !output.status.success() {
            return Err(ServerError::PlatformError(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl StatusProvider for WindowsStatusProvider {
    fn get_state_file_path(&self) -> Option<&Path> {
        Some(&self.paths.state_file)
    }

    fn get_agent_status(&self) -> Result<AgentStatus> {
        let output = self
            .run_powershell("(Get-Service -Name WazuhSvc -ErrorAction SilentlyContinue).Status")?;
        Ok(if output.to_lowercase().contains("running") {
            AgentStatus::Active
        } else {
            AgentStatus::Inactive
        })
    }

    fn get_connection_status(&self) -> Result<ConnectionStatus> {
        if !matches!(self.get_agent_status()?, AgentStatus::Active) {
            return Ok(ConnectionStatus::Disconnected);
        }
        read_connection_from_state_file(&self.paths.state_file)
    }

    fn get_agent_version(&self) -> Result<String> {
        if let Ok(content) = fs::read_to_string(&self.paths.version_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("version").and_then(|v| v.as_str()) {
                    return Ok(v.to_string());
                }
            }
        }
        Ok("Unknown".to_string())
    }

    fn get_tray_version(&self) -> Result<String> {
        match fs::read_to_string(&self.paths.version_file) {
            Ok(raw) => Ok(raw.trim().to_string()),
            Err(_) => Ok("Unknown".to_string()),
        }
    }

    fn get_agent_groups(&self) -> Result<Vec<String>> {
        Ok(group_extractor::extract_groups(&self.paths.merged_mg).unwrap_or_default())
    }

    fn get_system_metrics(&self) -> Result<crate::models::SystemMetrics> {
        let mut sys = self
            .sys
            .lock()
            .map_err(|_| ServerError::PlatformError("Failed to lock system metrics".to_string()))?;

        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sys.refresh_memory();
        sys.refresh_cpu_all();

        let mut total_cpu: f32 = 0.0;
        let mut total_rss: u64 = 0;
        let mut found = false;
        let mut agentd_found = false;

        for process in sys.processes().values() {
            let name = process.name().to_string_lossy();
            if !crate::status_provider::WINDOWS_AGENT_PROCESSES.contains(&name.as_ref()) {
                continue;
            }

            let cmd_path = process
                .cmd()
                .first()
                .and_then(|c| c.to_str().map(|s| s.to_lowercase()));
            let matches_path = cmd_path
                .as_ref()
                .map(|p| {
                    crate::status_provider::WINDOWS_EXE_PREFIXES
                        .iter()
                        .any(|prefix| p.starts_with(prefix))
                })
                .unwrap_or(true);

            if !matches_path {
                continue;
            }

            debug!(process = %name, "Matched Wazuh process");
            total_cpu += process.cpu_usage();
            total_rss += process.memory();
            if name.as_ref() == "wazuh-agentd.exe" || name.as_ref() == "ossec-agentd.exe" {
                agentd_found = true;
            }
            found = true;
        }

        if !found {
            debug!("No Wazuh processes matched");
        }

        let cpu_count = sys.cpus().len() as f32;
        let cpu_usage = if found && cpu_count > 0.0 {
            total_cpu / cpu_count
        } else {
            0.0
        };

        let total_memory = sys.total_memory();
        let memory_usage = if total_memory > 0 {
            total_rss as f32 / total_memory as f32
        } else {
            0.0
        };

        Ok(crate::models::SystemMetrics {
            cpu_usage,
            memory_usage,
            total_memory,
            used_memory: total_rss,
            agent_found: found,
            agentd_found,
        })
    }
}
