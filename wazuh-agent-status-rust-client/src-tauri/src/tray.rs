use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, Emitter,
};
use crate::agent::{AgentManager, AgentStatus, ConnectionStatus};
#[cfg(not(target_os = "linux"))]
use tauri_plugin_positioner::{WindowExt, Position};


#[allow(dead_code)]
pub struct TrayMenuState<R: Runtime> {
    pub show_item: MenuItem<R>,
}

pub fn setup_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let agent_manager = app.state::<std::sync::Arc<AgentManager>>();
    let mut rx = agent_manager.subscribe();
    let initial_state = rx.borrow().clone();

    let status_text = match initial_state.status {
        AgentStatus::Active => "Agent: Active",
        AgentStatus::Inactive => "Agent: Inactive",
        AgentStatus::Unknown => "Agent: Unknown",
    };
    
    let conn_text = match initial_state.connection {
        ConnectionStatus::Connected => "Connection: Connected",
        ConnectionStatus::Disconnected => "Connection: Disconnected",
        ConnectionStatus::Unknown => "Connection: Unknown",
    };

    let status_i = MenuItem::with_id(app, "status", status_text, false, None::<&str>)?;
    let conn_i = MenuItem::with_id(app, "connection", conn_text, false, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    
    let update_i = MenuItem::with_id(app, "update", "Check for Updates", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    let show_i = MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>)?;
    let show_i_state = show_i.clone();
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[
        &status_i, &conn_i, &sep1, 
        &update_i, &sep2, 
        &show_i, &quit_i
    ])?;

    let status_i_clone = status_i.clone();
    let conn_i_clone = conn_i.clone();

    tauri::async_runtime::spawn(async move {
        while rx.changed().await.is_ok() {
            let state = rx.borrow().clone();
            
            let status_text = match state.status {
                AgentStatus::Active => "Agent: Active",
                AgentStatus::Inactive => "Agent: Inactive",
                AgentStatus::Unknown => "Agent: Unknown",
            };
            let _ = status_i_clone.set_text(status_text);
            
            let conn_text = match state.connection {
                ConnectionStatus::Connected => "Connection: Connected",
                ConnectionStatus::Disconnected => "Connection: Disconnected",
                ConnectionStatus::Unknown => "Connection: Unknown",
            };
            let _ = conn_i_clone.set_text(conn_text);
        }
    });

    let icon_bytes = include_bytes!("../icons/tray.png");
    let image = image::load_from_memory(icon_bytes).expect("Failed to load icon from memory");
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);

    let show_i_tray = show_i.clone();
    
    let _ = TrayIconBuilder::with_id("wazuh-status-v1")
        .tooltip("Wazuh Agent Status")
        .icon(icon)
        .menu(&menu)
        .on_menu_event(move |app: &AppHandle<R>, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let is_visible = window.is_visible().unwrap_or(false);
                    if is_visible {
                        let _ = window.hide();
                        let _ = show_i.set_text("Show Dashboard");
                    } else {
                        // Positioner requires tray position to be set by on_tray_icon_event.
                        // On Windows, this is unreliable, so we use BottomRight as a safe fallback.
                        #[cfg(target_os = "windows")]
                        let _ = window.move_window(Position::BottomRight);
                        #[cfg(target_os = "macos")]
                        let _ = window.move_window(Position::TrayCenter);
                        
                        let _ = window.set_decorations(true);
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = show_i.set_text("Hide Dashboard");
                    }
                }
            }
            "update" => {
                if let Some(window) = app.get_webview_window("main") {
                    #[cfg(target_os = "windows")]
                    let _ = window.move_window(Position::BottomRight);
                    #[cfg(target_os = "macos")]
                    let _ = window.move_window(Position::TrayCenter);

                    let _ = window.set_decorations(true);
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = show_i.set_text("Hide Dashboard");
                    let _ = window.emit("navigate-to-updates", ());
                }
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray: &tauri::tray::TrayIcon<R>, event| {
            // Update positioner with tray coordinates for non-linux systems
            #[cfg(not(target_os = "linux"))]
            let _ = tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let is_visible = window.is_visible().unwrap_or(false);
                    
                    if is_visible {
                        let _ = window.hide();
                        let _ = show_i_tray.set_text("Show Dashboard");
                    } else {
                        #[cfg(target_os = "windows")]
                        let _ = window.move_window(Position::BottomRight);
                        #[cfg(target_os = "macos")]
                        let _ = window.move_window(Position::TrayCenter);
                        
                        let _ = window.set_decorations(true);
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = show_i_tray.set_text("Hide Dashboard");
                    }
                }
            }
        })
        .build(app)?;


    // Store state for window event sync
    app.manage(TrayMenuState { show_item: show_i_state });

    Ok(())
}
