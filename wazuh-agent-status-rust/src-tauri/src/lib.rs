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

use std::sync::Mutex;
use sysinfo::{System, ProcessRefreshKind, RefreshKind, ProcessesToUpdate};

pub struct MetricsManager {
    sys: Mutex<System>,
}

impl MetricsManager {
    pub fn new() -> Self {
        let kind = ProcessRefreshKind::nothing().with_cpu();
        let sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(kind)
        );
        Self { sys: Mutex::new(sys) }
    }
}

#[tauri::command]
fn get_system_metrics(metrics_manager: State<'_, MetricsManager>) -> serde_json::Value {
    let mut sys = metrics_manager.sys.lock().unwrap();
    
    let kind = ProcessRefreshKind::nothing().with_cpu();
    // Refresh for current metrics
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, kind);
    
    let process_name = if cfg!(windows) { "wazuh-agent.exe" } else { "wazuh-agentd" };
    
    let mut cpu_sum = 0.0;
    let mut mem_sum = 0;
    let mut target_detected = false;

    use std::ffi::OsStr;
    let os_process_name = OsStr::new(process_name);

    for process in sys.processes_by_exact_name(os_process_name) {
        cpu_sum += process.cpu_usage();
        mem_sum += process.memory(); // memory() is in bytes in 0.38
        target_detected = true;
    }

    if !target_detected {
        return serde_json::json!({
            "cpu_usage": 0.0,
            "memory_usage": 0.0,
            "total_memory": sys.total_memory(),
            "used_memory": 0,
            "agent_running": false
        });
    }
    
    serde_json::json!({
        "cpu_usage": cpu_sum,
        "memory_usage": (mem_sum as f64 / sys.total_memory() as f64) * 100.0,
        "total_memory": sys.total_memory(),
        "used_memory": mem_sum,
        "agent_running": true
    })
}

#[tauri::command]
fn minimize_window(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn hide_window(window: tauri::Window) {
    let _ = window.hide();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init());

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
            
            let agent_manager = AgentManager::new(config.clone());
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
