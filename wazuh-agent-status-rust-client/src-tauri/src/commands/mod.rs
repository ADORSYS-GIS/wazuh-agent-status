pub mod ai_commands;

use crate::agent::{AgentManager, AgentState, AgentStatus};
use crate::config::AppConfig;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Emitter, State};

#[tauri::command]
pub fn get_agent_status(manager: State<'_, Arc<AgentManager>>) -> AgentState {
    manager.get_state()
}

#[tauri::command]
pub fn get_config(config: State<'_, AppConfig>) -> AppConfig {
    config.inner().clone()
}



#[cfg(target_os = "windows")]
#[tauri::command]
pub fn notify_update_available(
    app: tauri::AppHandle,
    current_version: String,
    latest_version: String,
) {
    use tauri::Manager;
    use tauri_plugin_notification::NotificationExt;

    let _ = app
        .notification()
        .builder()
        .title("Update available")
        .body(format!(
            "A newer version is available: v{latest_version} (installed: v{current_version})."
        ))
        .show();

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn notify_update_available(
    _app: tauri::AppHandle,
    _current_version: String,
    _latest_version: String,
) {
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

fn gateway_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

fn generate_nonce() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let pid = std::process::id();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}-{:x}-{:x}", pid, nanos, count)
}

fn compute_hmac_proof(agent_key: &str, agent_id: &str, timestamp: u64, nonce: &str) -> String {
    let message = format!("{}:{}:{}", agent_id, timestamp, nonce);
    let mut mac =
        Hmac::<Sha256>::new_from_slice(agent_key.as_bytes()).expect("HMAC key length is valid");
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[tauri::command]
pub async fn fetch_compliance(
    config: State<'_, AppConfig>,
    manager: State<'_, Arc<AgentManager>>,
    agent_id: String,
    status_filter: Option<String>,
    mandatory: Option<bool>,
    category: Option<String>,
) -> Result<ComplianceReport, String> {
    let state = manager.get_state();
    if state.agent_key.is_empty() {
        return Err(
            "No agent authentication key available. The agent may not be enrolled.".to_string(),
        );
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let nonce = generate_nonce();
    let proof = compute_hmac_proof(&state.agent_key, &agent_id, timestamp, &nonce);

    let base = config.gateway_url.trim_end_matches('/');
    let url = format!("{}/agents/{}/compliance", base, agent_id);

    log::debug!("Fetching compliance from: {}", url);

    let client = gateway_http_client()?;

    let mut req = client
        .get(&url)
        .header("x-agent-proof", &proof)
        .header("x-agent-timestamp", timestamp.to_string())
        .header("x-agent-nonce", &nonce);

    if let Some(ref s) = status_filter {
        req = req.query(&[("status", s.as_str())]);
    }
    if let Some(m) = mandatory {
        req = req.query(&[("mandatory", m)]);
    }
    if let Some(ref c) = category {
        req = req.query(&[("category", c.as_str())]);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("Failed to fetch compliance: {}", e))?;

    if !resp.status().is_success() {
        let status_code = resp.status();
        let body = resp.text().await.unwrap_or_default();
        let error_message = if body.is_empty() {
            format!("Gateway returned HTTP {}", status_code)
        } else {
            // Try to extract error field from JSON response
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                val.get("error")
                    .and_then(|e| e.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(body)
            } else {
                body
            }
        };
        return Err(error_message);
    }

    let report: ComplianceReport = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse compliance report: {}", e))?;

    Ok(report)
}
