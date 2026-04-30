mod config;
mod agent;
mod tray;
mod commands;

use config::AppConfig;
use agent::AgentManager;
use tauri::Manager;
use anyhow::Context;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("tao", log::LevelFilter::Off)
                .build(),
        )
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
            
            let agent_manager = AgentManager::new(config.server_addr.clone());

            // Manage state
            app.manage(config);
            app.manage(agent_manager);

            tray::setup_tray(app.handle())
                .context("Failed to initialize system tray")?;
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                
                // Sync tray menu text
                let app_handle = window.app_handle();
                if let Some(tray_state) = app_handle.try_state::<tray::TrayMenuState<tauri::Wry>>() {
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
            commands::start_update
        ])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("Fatal error while running tauri application: {:?}", e);
        std::process::exit(1);
    }
}
