use tauri::{State, Emitter};
use crate::config::AppConfig;
use crate::agent::{AgentManager, AgentState, AgentStatus};
use std::sync::Arc;

#[tauri::command]
pub fn get_agent_status(manager: State<'_, Arc<AgentManager>>) -> AgentState {
    manager.get_state()
}

#[tauri::command]
pub fn get_config(config: State<'_, AppConfig>) -> AppConfig {
    config.inner().clone()
}

#[tauri::command]
pub async fn check_for_updates(manager: State<'_, Arc<AgentManager>>) -> Result<serde_json::Value, String> {
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
    is_prerelease: bool
) -> Result<(), String> {
    let mut rx = manager.run_update(is_prerelease).await.map_err(|e| e.to_string())?;
    
    tauri::async_runtime::spawn(async move {
        while let Some(line) = rx.recv().await {
            let _ = window.emit("update-log", line);
        }
    });

    Ok(())
}
