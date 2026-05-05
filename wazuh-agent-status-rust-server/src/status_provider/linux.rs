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
    sys:   std::sync::Mutex<System>,
}

impl LinuxStatusProvider {
    pub fn new(paths: AgentPaths) -> Self {
        let mut sys = System::new();
        // Initial refresh so we have something for first poll
        sys.refresh_all();
        Self { 
            paths,
            sys: std::sync::Mutex::new(sys),
        }
    }

    /// Determine whether the `wazuh-agentd` process is alive by calling the
    /// official `wazuh-control status` script. This ensures parity with the 
    /// local Wazuh management tools.
    fn is_agent_running(&self) -> bool {
        let control_path = self.paths.wazuh_control.clone();

        // Primary check: official wazuh-control utility
        match std::process::Command::new(&control_path).arg("status").output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let is_running = stdout.contains("wazuh-agentd is running");
                if !is_running {
                    tracing::info!("wazuh-control status says agent is NOT running");
                }
                is_running
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to execute wazuh-control status, falling back to process check");
                // Fallback: search process list if control utility is missing
                if let Ok(mut sys) = self.sys.lock() {
                    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                    let is_running = sys.processes().values().any(|p| p.name().to_string_lossy() == "wazuh-agentd");
                    if !is_running {
                        tracing::info!("Process list check confirms wazuh-agentd is NOT running");
                    }
                    is_running
                } else {
                    false
                }
            }
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
        // 1. Try VERSION.json first
        if let Ok(content) = fs::read_to_string(&self.paths.version_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("version").and_then(|v| v.as_str()) {
                    let version = v.to_string();
                    tracing::debug!(version = %version, path = %self.paths.version_json.display(), "Read agent version from VERSION.json");
                    return Ok(version);
                }
            }
        }

        // 2. Fallback to wazuh-control info
        let control_path = self.paths.state_file.parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|base| base.join("bin/wazuh-control"))
            .unwrap_or_else(|| std::path::PathBuf::from("/var/ossec/bin/wazuh-control"));

        if let Ok(output) = std::process::Command::new(&control_path).arg("info").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(v) = line.strip_prefix("WAZUH_VERSION=\"") {
                    let version = v.trim_matches('"').trim_start_matches('v').to_string();
                    tracing::info!(version = %version, "Read agent version from wazuh-control info");
                    return Ok(version);
                }
            }
        }

        tracing::warn!("Failed to detect Wazuh agent version via VERSION.json or wazuh-control");
        Ok("Unknown".to_string())
    }

    fn get_tray_version(&self) -> Result<String> {
        match fs::read_to_string(&self.paths.version_file) {
            Ok(raw) => {
                let v = raw.trim().to_string();
                tracing::debug!(version = %v, path = %self.paths.version_file.display(), "Read tray app version");
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
        let mut sys = self.sys.lock().map_err(|_| {
            ServerError::PlatformError("Failed to lock system metrics".to_string())
        })?;

        // Refresh all processes to find the Wazuh ones
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sys.refresh_memory();
        sys.refresh_cpu_all(); // Ensure core information is fresh for delta calculation

        let mut total_cpu: f32 = 0.0;
        let mut total_rss: u64 = 0;
        let mut found_names = Vec::new();

        for process in sys.processes().values() {
            let name = process.name().to_string_lossy();
            if crate::status_provider::UNIX_AGENT_PROCESSES.contains(&name.as_ref()) {
                let p_cpu = process.cpu_usage();
                total_cpu += p_cpu;
                total_rss += process.memory();
                found_names.push(format!("{} ({:.1}%)", name, p_cpu));
            }
        }


        let cpu_count = sys.cpus().len() as f32;
        let cpu_usage = if !found_names.is_empty() && cpu_count > 0.0 {
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
        })
    }
}
