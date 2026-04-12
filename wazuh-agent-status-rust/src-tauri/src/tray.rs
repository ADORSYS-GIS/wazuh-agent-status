use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
#[cfg(not(target_os = "linux"))]
use tauri_plugin_positioner::{WindowExt, Position};

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    let icon_bytes = include_bytes!("../icons/tray.png");
    let image = image::load_from_memory(icon_bytes).expect("Failed to load icon from memory");
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let icon = tauri::image::Image::new_owned(rgba.into_raw(), width, height);

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
                    // Optimized positioning:
                    // - macOS/Windows: Uses TrayCenter for professional placement near the icon.
                    // - Linux: Defaults to standard window manager placement for maximum stability.
                    #[cfg(not(target_os = "linux"))]
                    let _ = window.as_ref().window().move_window(Position::TrayCenter); 
                    
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray: &tauri::tray::TrayIcon<R>, event| {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        // Position window near tray on macOS and Windows
                        #[cfg(not(target_os = "linux"))]
                        let _ = window.as_ref().window().move_window(Position::TrayCenter);
                        
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
