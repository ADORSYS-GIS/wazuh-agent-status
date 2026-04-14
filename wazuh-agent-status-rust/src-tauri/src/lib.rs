mod config;
mod agent;
mod tray;

use config::AppConfig;
use agent::{AgentManager, AgentStatus};
use tauri::Manager;
use tauri::State;

#[tauri::command]
fn get_agent_status(manager: State<'_, AgentManager>) -> AgentStatus {
    manager.check_status()
}

#[tauri::command]
fn get_config(config: State<'_, AppConfig>) -> AppConfig {
    config.inner().clone()
}

#[tauri::command]
async fn check_for_update(config: State<'_, AppConfig>) -> Result<config::UpdateCheckResult, String> {
    config.check_for_updates().await
}

#[tauri::command]
async fn perform_update(app: tauri::AppHandle, config: State<'_, AppConfig>, download_url: String) -> Result<(), String> {
    config.apply_update(download_url).await?;
    app.restart();
    #[allow(unreachable_code)]
    Ok(())
}

pub struct MetricsManager;

impl MetricsManager {
    pub fn new() -> Self {
        Self
    }
}

#[tauri::command]
fn get_system_metrics(_metrics_manager: State<'_, MetricsManager>) -> serde_json::Value {
    // Return hardcoded/simulated metrics
    serde_json::json!({
        "cpu_usage": 1.2,
        "memory_usage": 0.45,
        "total_memory": 16000000000u64,
        "used_memory": 72000000,
        "agent_running": true
    })
}

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn hide_window<R: tauri::Runtime>(window: tauri::Window<R>, tray_state: tauri::State<'_, tray::TrayMenuState<R>>) {
    let _ = window.hide();
    let _ = tray_state.show_item.set_text("Show Dashboard");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init());

    #[cfg(not(target_os = "linux"))]
    {
        builder = builder.plugin(tauri_plugin_positioner::init());
    }

    builder
        .setup(|app| {
            // Initialize Managers
            let config = AppConfig::load(app.handle())
                .map_err(|e| {
                    let err: Box<dyn std::error::Error> = e.into();
                    tauri::Error::Setup(err.into())
                })?;
            
            let agent_manager = AgentManager::new();
            let metrics_manager = MetricsManager::new();

            // Manage state
            app.manage(config);
            app.manage(agent_manager);
            app.manage(metrics_manager);

            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![get_agent_status, get_config, get_system_metrics, minimize_window, hide_window, check_for_update, perform_update])
        .run(tauri::generate_context!())
        .expect("Fatal error while running tauri application");
}
