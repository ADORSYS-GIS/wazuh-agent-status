use crate::agent::{AgentManager, AgentState, AgentStatus};
use crate::config::AppConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, State};

#[tauri::command]
pub fn get_agent_status(manager: State<'_, Arc<AgentManager>>) -> AgentState {
    manager.get_state()
}

#[tauri::command]
pub fn get_config(config: State<'_, AppConfig>) -> AppConfig {
    config.inner().clone()
}

#[tauri::command]
pub async fn check_for_updates(
    manager: State<'_, Arc<AgentManager>>,
) -> Result<serde_json::Value, String> {
    manager.check_updates().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_system_metrics(manager: State<'_, Arc<AgentManager>>) -> serde_json::Value {
    let state = manager.get_state();
    serde_json::json!({
        "cpu_usage": state.metrics.cpu_usage,
        "memory_usage": state.metrics.memory_usage,
        "total_memory": state.metrics.total_memory,
        "used_memory": state.metrics.used_memory,
        "agent_running": matches!(state.status, AgentStatus::Active)
    })
}

#[tauri::command]
pub async fn start_update(
    window: tauri::Window,
    manager: State<'_, Arc<AgentManager>>,
    is_prerelease: bool,
) -> Result<(), String> {
    let mut rx = manager
        .run_update(is_prerelease)
        .await
        .map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn(async move {
        while let Some(line) = rx.recv().await {
            let _ = window.emit("update-log", line);
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn start_log_stream(
    window: tauri::Window,
    manager: State<'_, Arc<AgentManager>>,
) -> Result<(), String> {
    let mut rx = manager.stream_logs().await.map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn(async move {
        while let Some(line) = rx.recv().await {
            let _ = window.emit("log-line", line);
        }
    });

    Ok(())
}

// ── SCA / Compliance ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResult {
    pub check_id: u32,
    pub title: String,
    pub status: String,
    pub mandatory: bool,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCategory {
    pub name: String,
    pub status: String,
    pub passed_count: usize,
    pub failed_count: usize,
    pub untested_count: usize,
    pub checks: Vec<ComplianceCheckResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub agent_id: String,
    pub agent_name: String,
    pub os: String,
    pub score: u32,
    pub compliance_status: String,
    pub total_passed_count: usize,
    pub total_failed_count: usize,
    pub total_untested_count: usize,
    pub categories: Vec<ComplianceCategory>,
}

/// Create a shared reqwest HTTP client with safe defaults for gateway calls.
fn gateway_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // Wazuh self-signed certs
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Fetch compliance report for an agent from the Wazuh Gateway.
#[tauri::command]
pub async fn fetch_compliance(
    config: State<'_, AppConfig>,
    agent_id: String,
    status_filter: Option<String>,
    mandatory: Option<bool>,
    category: Option<String>,
) -> Result<ComplianceReport, String> {
    let base = config.gateway_url.trim_end_matches('/');
    let url = format!("{}/agents/{}/compliance", base, agent_id);

    log::info!("Fetching compliance from: {}", url);

    let client = gateway_http_client()?;

    // Build query params using reqwest's built-in API (handles encoding)
    let mut req = client.get(&url);
    let mut has_params = false;

    if let Some(ref s) = status_filter {
        req = req.query(&[("status", s.as_str())]);
        has_params = true;
    }
    if let Some(m) = mandatory {
        req = req.query(&[("mandatory", m)]);
        has_params = true;
    }
    if let Some(ref c) = category {
        req = req.query(&[("category", c.as_str())]);
        has_params = true;
    }

    if has_params {
        log::info!("With query parameters: status={:?}, mandatory={:?}, category={:?}",
            status_filter, mandatory, category);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("Failed to fetch compliance: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Gateway returned {}: {}", status, body));
    }

    let report: ComplianceReport = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse compliance report: {}", e))?;

    Ok(report)
}
