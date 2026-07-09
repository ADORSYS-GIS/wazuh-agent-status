//! Shared Unix system-metrics collection for Linux & macOS providers.

use std::fs;
use sysinfo::System;

use crate::models::SystemMetrics;
use crate::status_provider::UNIX_AGENT_PROCESSES;

/// Executable-path prefixes that identify a local Wazuh installation.
pub(crate) const WAZUH_EXE_PREFIXES: &[&str] = &["/var/ossec/", "/Library/Ossec/"];

/// On Linux, `/proc/[pid]/status` exposes `Tgid` (thread group ID). If it
/// differs from `Pid`, the entry is a thread, not a process.
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

/// Collect system metrics from Wazuh agent processes on Unix platforms.
///
/// Uses `process.cmd()` (world-readable `/proc/[pid]/cmdline`) instead of
/// `process.exe()` (permission-restricted `/proc/[pid]/exe`) for path matching.
pub(crate) fn collect_unix_system_metrics(sys: &System) -> SystemMetrics {
    let mut total_cpu: f32 = 0.0;
    let mut total_rss: u64 = 0;
    let mut found_names = Vec::new();
    let mut agentd_found = false;

    for process in sys.processes().values() {
        let name = process.name().to_string_lossy();
        if !UNIX_AGENT_PROCESSES.contains(&name.as_ref()) {
            continue;
        }

        // Use cmd() (world-readable /proc/[pid]/cmdline) instead of exe()
        // (/proc/[pid]/exe, which needs matching permissions).
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
            .unwrap_or(true); // fallback: if cmd() is empty, accept name-only match

        if !matches_path {
            tracing::trace!(
                "Skipping process '{}' (PID {}) because its cmd path '{}' does not match Wazuh prefixes",
                name,
                process.pid(),
                cmd_path.as_deref().unwrap_or("<unknown>"),
            );
            continue;
        }

        // Skip threads so we count each Wazuh daemon only once.
        if is_thread(process.pid().as_u32()) {
            tracing::trace!(
                "Skipping thread '{}' (TID {}, TGID via /proc/{}/status)",
                name,
                process.pid(),
                process.pid(),
            );
            continue;
        }

        let p_cpu = process.cpu_usage();
        total_cpu += p_cpu;
        total_rss += process.memory();
        if name.as_ref() == "wazuh-agentd" {
            agentd_found = true;
        }
        found_names.push(format!("PID {} {} ({:.1}%)", process.pid(), name, p_cpu));
    }

    if !found_names.is_empty() {
        tracing::debug!("Found {} Wazuh processes", found_names.len());
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

    SystemMetrics {
        cpu_usage,
        memory_usage,
        total_memory,
        used_memory: total_rss,
        agent_found: !found_names.is_empty(),
        agentd_found,
    }
}
