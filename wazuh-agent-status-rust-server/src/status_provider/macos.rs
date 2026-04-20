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
use sysinfo::System;

use crate::config::AgentPaths;
use crate::errors::{Result, ServerError};
use crate::group_extractor;
use crate::models::{AgentStatus, ConnectionStatus};
use crate::status_provider::StatusProvider;

pub struct MacosStatusProvider {
    paths: AgentPaths,
    sys:   std::sync::Mutex<System>,
}

impl MacosStatusProvider {
    pub fn new(paths: AgentPaths) -> Self {
        let mut sys = System::new();
        sys.refresh_all();
        Self { 
            paths,
            sys: std::sync::Mutex::new(sys),
        }
    }

    /// Determine whether `wazuh-agentd` is alive by running `wazuh-control status`.
    fn is_agent_running(&self) -> bool {
        let control_path = self.paths.state_file.parent() // var/run/
            .and_then(|p| p.parent())                    // var/
            .and_then(|p| p.parent())                    // base/
            .map(|base| base.join("bin/wazuh-control"))
            .unwrap_or_else(|| std::path::PathBuf::from("/Library/Ossec/bin/wazuh-control"));

        // Primary check: official wazuh-control utility
        match Command::new(&control_path).arg("status").output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("wazuh-agentd is running") {
                    return true;
                }
                // If wazuh-control ran successfully but didn't say it's running, 
                // trust it and return false instead of falling back.
                if output.status.success() || stdout.contains("wazuh-agentd is stopped") {
                    return false;
                }
            }
            Err(_) => {
                // wazuh-control not found or failed to spawn
            }
        }

        false
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
        // 1. Try VERSION.json first
        if let Ok(content) = fs::read_to_string(&self.paths.version_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = json.get("version").and_then(|v| v.as_str()) {
                    return Ok(v.to_string());
                }
            }
        }

        // 2. Fallback to wazuh-control info
        let control_path = self.paths.state_file.parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|base| base.join("bin/wazuh-control"))
            .unwrap_or_else(|| std::path::PathBuf::from("/Library/Ossec/bin/wazuh-control"));

        if let Ok(output) = Command::new(&control_path).arg("info").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(v) = line.strip_prefix("WAZUH_VERSION=\"") {
                    return Ok(v.trim_matches('"').trim_start_matches('v').to_string());
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
        match group_extractor::extract_groups(&self.paths.merged_mg) {
            Ok(groups) => Ok(groups),
            Err(_) => Ok(Vec::new()), // No groups if agent is not installed
        }
    }

    fn get_system_metrics(&self) -> Result<crate::models::SystemMetrics> {
        let mut sys = self.sys.lock().map_err(|_| {
            ServerError::PlatformError("Failed to lock system metrics".to_string())
        })?;

        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sys.refresh_memory();
        sys.refresh_cpu_all();

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

        if !found_names.is_empty() {
            tracing::debug!("Found Wazuh processes: {}", found_names.join(", "));
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
