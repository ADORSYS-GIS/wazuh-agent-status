use crate::agent::{AgentManager, AgentState, AgentStatus, ConnectionStatus};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};
#[cfg(not(target_os = "linux"))]
use tauri_plugin_positioner::{Position, WindowExt};

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

    let menu = Menu::with_items(
        app,
        &[
            &status_i, &conn_i, &sep1, &update_i, &sep2, &show_i, &quit_i,
        ],
    )?;

    let show_i_tray = show_i.clone();

    let initial_state_val = initial_state.clone();
    let tray_icon = TrayIconBuilder::with_id("wazuh-status-v1")
        .tooltip("Wazuh Agent Status")
        .icon(get_status_icon(&initial_state_val))
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
            } = event
            {
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

    let status_i_clone = status_i.clone();
    let conn_i_clone = conn_i.clone();
    let tray_icon_handle = tray_icon.clone();

    tauri::async_runtime::spawn(async move {
        while rx.changed().await.is_ok() {
            let state = rx.borrow().clone();

            let (status_dot, status_text) = match state.status {
                AgentStatus::Active => ("🟢", "Agent: Active"),
                AgentStatus::Inactive => ("🔴", "Agent: Inactive"),
                AgentStatus::Unknown => ("⚪", "Agent: Unknown"),
            };
            let _ = status_i_clone.set_text(format!("{} {}", status_dot, status_text));

            let (conn_dot, conn_text) = match state.connection {
                ConnectionStatus::Connected => ("🟢", "Connection: Connected"),
                ConnectionStatus::Disconnected => ("🔴", "Connection: Disconnected"),
                ConnectionStatus::Unknown => ("⚪", "Connection: Unknown"),
            };
            let _ = conn_i_clone.set_text(format!("{} {}", conn_dot, conn_text));

            // Update tray icon
            let _ = tray_icon_handle.set_icon(Some(get_status_icon(&state)));
            log::info!(
                "Updated tray icon and menu for status: {:?}, connection: {:?}",
                state.status,
                state.connection
            );
        }
    });

    // Store state for window event sync
    app.manage(TrayMenuState {
        show_item: show_i_state,
    });

    Ok(())
}

fn get_status_icon(state: &AgentState) -> tauri::image::Image<'_> {
    let icon_bytes = include_bytes!("../icons/tray.png");
    let mut img = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon from memory")
        .to_rgba8();
    let (width, height) = img.dimensions();

    let color = match (&state.status, &state.connection) {
        (AgentStatus::Active, ConnectionStatus::Connected) => image::Rgba([0, 215, 0, 255]), // Bright Green
        (AgentStatus::Inactive, _) | (_, ConnectionStatus::Disconnected) => {
            image::Rgba([215, 0, 0, 255])
        } // Red
        _ => image::Rgba([128, 128, 128, 255]), // Gray for Unknown/other
    };

    // Draw a status dot in the bottom-right corner
    let dot_radius = 4;
    let center_x = (width - dot_radius - 2) as i32;
    let center_y = (height - dot_radius - 2) as i32;

    for x in (center_x - dot_radius as i32)..(center_x + dot_radius as i32) {
        for y in (center_y - dot_radius as i32)..(center_y + dot_radius as i32) {
            let dx = x - center_x;
            let dy = y - center_y;
            if dx * dx + dy * dy <= (dot_radius * dot_radius) as i32 {
                if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    tauri::image::Image::new_owned(img.into_raw(), width, height)
}
