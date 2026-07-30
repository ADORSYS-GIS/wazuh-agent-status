use std::fs;
use std::path::Path;
use sysinfo::System;

use crate::config::AgentPaths;
use crate::errors::{Result, ServerError};
use crate::group_extractor;
use crate::models::{AgentStatus, ConnectionStatus, SystemMetrics};
use crate::status_provider::{
    StatusProvider, UNIX_AGENT_PROCESSES, read_connection_from_state_file,
};

pub(crate) const WAZUH_EXE_PREFIXES: &[&str] = &["/var/ossec/", "/Library/Ossec/"];

pub struct UnixStatusProvider {
    paths: AgentPaths,
    sys: std::sync::Mutex<System>,
}

impl UnixStatusProvider {
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
            sys.processes()
                .values()
                .any(|p| p.name().to_string_lossy() == "wazuh-agentd")
        } else {
            false
        }
    }
}

impl StatusProvider for UnixStatusProvider {
    fn get_state_file_path(&self) -> Option<&Path> {
        Some(&self.paths.state_file)
    }

    fn get_agent_status(&self) -> Result<AgentStatus> {
        Ok(if self.is_agent_running() {
            AgentStatus::Active
        } else {
            AgentStatus::Inactive
        })
    }

    fn get_connection_status(&self) -> Result<ConnectionStatus> {
        if !self.is_agent_running() {
            return Ok(ConnectionStatus::Disconnected);
        }
        read_connection_from_state_file(&self.paths.state_file)
    }

    fn get_agent_version(&self) -> Result<String> {
        match fs::read_to_string(&self.paths.version_json) {
            Ok(content) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
                    && let Some(v) = json.get("version").and_then(|v| v.as_str())
                {
                    return Ok(v.to_string());
                }
                Ok("Unknown".to_string())
            }
            Err(_) => Ok("Unknown".to_string()),
        }
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

    fn get_system_metrics(&self) -> Result<SystemMetrics> {
        let mut sys = self
            .sys
            .lock()
            .map_err(|_| ServerError::PlatformError("Failed to lock system metrics".to_string()))?;

        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        sys.refresh_memory();
        sys.refresh_cpu_all();

        Ok(collect_unix_system_metrics(&sys))
    }
}

#[cfg(target_os = "linux")]
fn is_thread(pid: u32) -> bool {
    if let Ok(content) = fs::read_to_string(format!("/proc/{pid}/status")) {
        let mut tgid = pid;
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("Tgid:") {
                tgid = rest.trim().parse().unwrap_or(pid);
                break;
            }
        }
        tgid != pid
    } else {
        false
    }
}

#[cfg(not(target_os = "linux"))]
fn is_thread(_pid: u32) -> bool {
    false
}

pub(crate) fn collect_unix_system_metrics(sys: &System) -> SystemMetrics {
    let mut total_cpu: f32 = 0.0;
    let mut total_rss: u64 = 0;
    let mut found = false;
    let mut agentd_found = false;

    for process in sys.processes().values() {
        let name = process.name().to_string_lossy();
        if !UNIX_AGENT_PROCESSES.contains(&name.as_ref()) {
            continue;
        }

        let cmd_path = process
            .cmd()
            .first()
            .and_then(|c| c.to_str().map(String::from));
        let matches_path = cmd_path
            .as_ref()
            .map(|p| {
                WAZUH_EXE_PREFIXES
                    .iter()
                    .any(|prefix| p.starts_with(prefix))
            })
            .unwrap_or(true);

        if !matches_path || is_thread(process.pid().as_u32()) {
            continue;
        }

        total_cpu += process.cpu_usage();
        total_rss += process.memory();
        if name.as_ref() == "wazuh-agentd" {
            agentd_found = true;
        }
        found = true;
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

    SystemMetrics {
        cpu_usage,
        memory_usage,
        total_memory,
        used_memory: total_rss,
        agent_found: found,
        agentd_found,
    }
}
