use std::sync::Arc;
use std::time::Duration;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tokio::sync::Mutex;
use tokio::time::interval;
use wazuh_status_common::WazuhClient;
use wazuh_status_proto_build::wazuh_status::{UpdateState, VersionState};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(serde::Serialize)]
struct StatusDto {
    agent_state: String,
    connection_state: String,
}

#[tauri::command]
async fn daemon_status(client: tauri::State<'_, WazuhClient>) -> Result<StatusDto, String> {
    let status = client.get_status().await.map_err(|err| err.to_string())?;
    Ok(StatusDto {
        agent_state: format!("{:?}", status.agent_state()),
        connection_state: format!("{:?}", status.connection_state()),
    })
}

fn format_agent_state(state: wazuh_status_proto_build::wazuh_status::AgentState) -> String {
    match state {
        wazuh_status_proto_build::wazuh_status::AgentState::Active => {
            "Agent: Active".to_string()
        }
        wazuh_status_proto_build::wazuh_status::AgentState::Inactive => {
            "Agent: Inactive".to_string()
        }
        _ => "Agent: Unknown".to_string(),
    }
}

fn format_connection_state(
    state: wazuh_status_proto_build::wazuh_status::ConnectionState,
) -> String {
    match state {
        wazuh_status_proto_build::wazuh_status::ConnectionState::Connected => {
            "Connection: Connected".to_string()
        }
        wazuh_status_proto_build::wazuh_status::ConnectionState::Disconnected => {
            "Connection: Disconnected".to_string()
        }
        _ => "Connection: Unknown".to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_opener::init())
        .manage(WazuhClient::default())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let status_item = MenuItem::with_id(app, "status", "Agent: Unknown", false, None::<&str>)?;
            let connection_item =
                MenuItem::with_id(app, "connection", "Connection: Unknown", false, None::<&str>)?;
            let pause_item = MenuItem::with_id(app, "pause", "Pause", true, None::<&str>)?;
            let restart_item = MenuItem::with_id(app, "restart", "Restart", true, None::<&str>)?;
            let update_item = MenuItem::with_id(app, "update", "Update", true, None::<&str>)?;
            let version_item = MenuItem::with_id(app, "version", "Version: Unknown", false, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &status_item,
                    &connection_item,
                    &pause_item,
                    &restart_item,
                    &update_item,
                    &version_item,
                    &quit_i,
                ],
            )?;
            let menu_image_update = Image::from_bytes(include_bytes!("../icons/tray.png")).unwrap();

            let _tray = TrayIconBuilder::new()
                .icon(menu_image_update)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .build(app)?;

            let version_state = Arc::new(Mutex::new(VersionState::Unknown));
            let update_state = Arc::new(Mutex::new(UpdateState::Unknown));
            let daemon_online = Arc::new(Mutex::new(false));

            let status_item_clone = status_item.clone();
            let connection_item_clone = connection_item.clone();
            let pause_item_clone = pause_item.clone();
            let restart_item_clone = restart_item.clone();
            let update_item_clone = update_item.clone();
            let daemon_online_clone = Arc::clone(&daemon_online);
            let client = app.state::<WazuhClient>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let mut ticker = interval(Duration::from_secs(5));
                loop {
                    match client.get_status().await {
                        Ok(status) => {
                            {
                                let mut guard = daemon_online_clone.lock().await;
                                *guard = true;
                            }
                            let _ = status_item_clone
                                .set_text(format_agent_state(status.agent_state()));
                            let _ = connection_item_clone
                                .set_text(format_connection_state(status.connection_state()));
                            let _ = pause_item_clone.set_enabled(true);
                            let _ = restart_item_clone.set_enabled(true);
                        }
                        Err(_) => {
                            {
                                let mut guard = daemon_online_clone.lock().await;
                                *guard = false;
                            }
                            let _ = status_item_clone.set_text("Daemon: Unreachable");
                            let _ = connection_item_clone.set_text("Connection: Unknown");
                            let _ = pause_item_clone.set_enabled(false);
                            let _ = restart_item_clone.set_enabled(false);
                            let _ = update_item_clone.set_enabled(false);
                        }
                    }
                    ticker.tick().await;
                }
            });

            let version_item_clone = version_item.clone();
            let update_item_clone = update_item.clone();
            let version_state_clone = Arc::clone(&version_state);
            let daemon_online_clone = Arc::clone(&daemon_online);
            let client = app.state::<WazuhClient>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let mut ticker = interval(Duration::from_secs(60 * 60 * 4));
                loop {
                    match client.check_version().await {
                        Ok(reply) => {
                            let state = reply.version_state();
                            let version = reply.version;
                            {
                                let mut guard = version_state_clone.lock().await;
                                *guard = state;
                            }
                            let _ = version_item_clone.set_text(format!("Version: {version}"));
                            let online = *daemon_online_clone.lock().await;
                            match state {
                                VersionState::Outdated if online => {
                                    let _ = update_item_clone.set_text("Update");
                                    let _ = update_item_clone.set_enabled(true);
                                }
                                VersionState::UpToDate => {
                                    let _ = update_item_clone.set_text("Up to date");
                                    let _ = update_item_clone.set_enabled(false);
                                }
                                _ => {
                                    let _ = update_item_clone.set_text("Update");
                                    let _ = update_item_clone.set_enabled(false);
                                }
                            }
                        }
                        Err(_) => {
                            let _ = version_item_clone.set_text("Version: Unknown");
                            let _ = update_item_clone.set_enabled(false);
                        }
                    }
                    ticker.tick().await;
                }
            });

            let update_item_clone = update_item.clone();
            let update_state_clone = Arc::clone(&update_state);
            let version_state_for_update = Arc::clone(&version_state);
            let daemon_online_for_update = Arc::clone(&daemon_online);
            let client = app.state::<WazuhClient>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let mut ticker = interval(Duration::from_secs(5));
                loop {
                    match client.get_update_status().await {
                        Ok(reply) => {
                            let state = reply.update_state();
                            let mut guard = update_state_clone.lock().await;
                            *guard = state;
                            if matches!(state, UpdateState::InProgress) {
                                let _ = update_item_clone.set_text("Updating...");
                                let _ = update_item_clone.set_enabled(false);
                            }
                            if matches!(state, UpdateState::Idle) {
                                let version_state = version_state_for_update.lock().await;
                                let online = *daemon_online_for_update.lock().await;
                                match *version_state {
                                    VersionState::Outdated if online => {
                                        let _ = update_item_clone.set_text("Update");
                                        let _ = update_item_clone.set_enabled(true);
                                    }
                                    VersionState::UpToDate => {
                                        let _ = update_item_clone.set_text("Up to date");
                                        let _ = update_item_clone.set_enabled(false);
                                    }
                                    _ => {
                                        let _ = update_item_clone.set_text("Update");
                                        let _ = update_item_clone.set_enabled(false);
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            let mut guard = update_state_clone.lock().await;
                            *guard = UpdateState::Unknown;
                        }
                    }
                    ticker.tick().await;
                }
            });

            let update_item_clone = update_item.clone();
            let version_state_clone = Arc::clone(&version_state);
            let daemon_online_clone = Arc::clone(&daemon_online);
            let client = app.state::<WazuhClient>().inner().clone();
            app.on_menu_event(move |app, event| {
                match event.id().as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "pause" => {
                        let online = daemon_online_clone.clone();
                        let client = client.clone();
                        tauri::async_runtime::spawn(async move {
                            if *online.lock().await {
                                let _ = client.pause().await;
                            }
                        });
                    }
                    "restart" => {
                        let online = daemon_online_clone.clone();
                        let client = client.clone();
                        tauri::async_runtime::spawn(async move {
                            if *online.lock().await {
                                let _ = client.restart().await;
                            }
                        });
                    }
                    "update" => {
                        let update_item = update_item_clone.clone();
                        let version_state = Arc::clone(&version_state_clone);
                        let online = daemon_online_clone.clone();
                        let client = client.clone();
                        tauri::async_runtime::spawn(async move {
                            if *online.lock().await
                                && matches!(*version_state.lock().await, VersionState::Outdated)
                            {
                                let _ = update_item.set_text("Updating...");
                                let _ = update_item.set_enabled(false);
                                let _ = client.start_update().await;
                            }
                        });
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, daemon_status]);

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }));
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{format_agent_state, format_connection_state};
    use wazuh_status_proto_build::wazuh_status::{AgentState, ConnectionState};

    #[test]
    fn format_agent_state_texts() {
        assert_eq!(format_agent_state(AgentState::Active), "Agent: Active");
        assert_eq!(format_agent_state(AgentState::Inactive), "Agent: Inactive");
        assert_eq!(format_agent_state(AgentState::Unknown), "Agent: Unknown");
    }

    #[test]
    fn format_connection_state_texts() {
        assert_eq!(
            format_connection_state(ConnectionState::Connected),
            "Connection: Connected"
        );
        assert_eq!(
            format_connection_state(ConnectionState::Disconnected),
            "Connection: Disconnected"
        );
        assert_eq!(
            format_connection_state(ConnectionState::Unknown),
            "Connection: Unknown"
        );
    }
}
