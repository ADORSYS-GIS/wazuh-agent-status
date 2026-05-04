//! Windows status provider.
//!
//! Agent status is queried via PowerShell `Get-Service` because Windows has no
//! equivalent of a Unix PID file that can be checked without elevated rights.
//! All other data (connection state, version, groups) is read directly from
//! the file system — no elevated privileges required for those reads.

use std::fs;
use std::process::Command;
use sysinfo::System;

use crate::config::AgentPaths;
use crate::errors::{Result, ServerError};
use crate::group_extractor;
use crate::models::{AgentStatus, ConnectionStatus};
use crate::status_provider::StatusProvider;

pub struct WindowsStatusProvider {
    paths: AgentPaths,
    sys:   std::sync::Mutex<System>,
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
        // Try VERSION.json first
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
        match group_extractor::extract_groups(&self.paths.merged_mg) {
            Ok(groups) => Ok(groups),
            Err(_) => Ok(Vec::new()),
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
        let mut found = false;

        for process in sys.processes().values() {
            let name = process.name().to_string_lossy();
            if crate::status_provider::WINDOWS_AGENT_PROCESSES.contains(&name.as_ref()) {
                total_cpu += process.cpu_usage();
                total_rss += process.memory();
                found = true;
            }
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
        })
    }
}
