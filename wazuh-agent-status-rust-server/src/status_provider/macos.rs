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

        // 1. Primary check: official wazuh-control utility
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
                // wazuh-control not found or failed to spawn; proceed to fallback
            }
        }

        // 2. Fallback: Check if the PID file exists and the process in it is actually wazuh-agentd.
        // This prevents false positives from stale PID files where the PID was reused.
        self.read_pid()
            .map(|pid| {
                Command::new("ps")
                    .args(["-p", &pid.to_string(), "-o", "comm="])
                    .output()
                    .map(|o| {
                        let comm = String::from_utf8_lossy(&o.stdout);
                        o.status.success() && comm.contains("wazuh-agentd")
                    })
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
        match fs::read_to_string(&self.paths.version_file) {
            Ok(raw) => Ok(raw.trim().to_string()),
            Err(_) => Ok("Not Installed".to_string()),
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

        sys.refresh_memory();
        sys.refresh_cpu_all();

        let total_memory = sys.total_memory();
        let used_memory  = sys.used_memory();
        let memory_usage = if total_memory > 0 {
            used_memory as f32 / total_memory as f32
        } else {
            0.0
        };

        let cpu_usage = sys.global_cpu_usage();

        Ok(crate::models::SystemMetrics {
            cpu_usage,
            memory_usage,
            total_memory,
            used_memory,
        })
    }
}
