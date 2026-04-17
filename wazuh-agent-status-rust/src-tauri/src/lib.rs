mod agent;
mod commands;
mod config;
mod tray;

use agent::AgentManager;
use anyhow::Context;
use config::AppConfig;
use tauri::Manager;

pub struct MetricsManager;

impl MetricsManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Setup positioner only on non-linux
            #[cfg(not(target_os = "linux"))]
            {
                app.handle().plugin(tauri_plugin_positioner::init())?;
            }

            // Hide dock icon on macOS
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Initialize Managers
            let config = AppConfig::load(app.handle())
                .map_err(|e| anyhow::anyhow!(e))
                .context("Failed to load application configuration")?;

            let agent_manager = AgentManager::new();
            let metrics_manager = MetricsManager::new();

            // Manage state
            app.manage(config);
            app.manage(agent_manager);
            app.manage(metrics_manager);

            tray::setup_tray(app.handle()).context("Failed to initialize system tray")?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();

                // Sync tray menu text
                let app_handle = window.app_handle();
                if let Some(tray_state) = app_handle.try_state::<tray::TrayMenuState<tauri::Wry>>()
                {
                    let _ = tray_state.show_item.set_text("Show Dashboard");
                }

                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_agent_status,
            commands::get_config,
            commands::get_system_metrics,
            commands::check_for_updates,
            commands::perform_update
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("Fatal error while running tauri application: {:?}", e);
        std::process::exit(1);
    }
}
