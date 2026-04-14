use tauri::{State, AppHandle};
use crate::config::{AppConfig, UpdateCheckResult};
use crate::agent::{AgentManager, AgentStatus};

#[tauri::command]
pub fn get_agent_status(manager: State<'_, AgentManager>) -> AgentStatus {
    manager.check_status()
}

#[tauri::command]
pub fn get_config(config: State<'_, AppConfig>) -> AppConfig {
    config.inner().clone()
}

#[tauri::command]
pub async fn check_for_updates(config: State<'_, AppConfig>) -> Result<UpdateCheckResult, String> {
    config.check_for_updates().await
}

#[tauri::command]
pub async fn perform_update(app: AppHandle, config: State<'_, AppConfig>, download_url: String) -> Result<(), String> {
    config.apply_update(download_url).await?;
    app.restart();
    #[allow(unreachable_code)]
    Ok(())
}

#[tauri::command]
pub fn get_system_metrics() -> serde_json::Value {
    // Return hardcoded/simulated metrics
    serde_json::json!({
        "cpu_usage": 1.2,
        "memory_usage": 0.45,
        "total_memory": 16000000000u64,
        "used_memory": 72000000,
        "agent_running": true
    })
}
