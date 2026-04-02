use crate::assets::Assets;
use anyhow::{Context, Result};
use image::GenericImageView;
use std::sync::Mutex;
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem, IconMenuItem, PredefinedMenuItem, Icon as MenuIcon},
};

pub struct TrayManager {
    tray_icon: TrayIcon,
    icon_active: MenuIcon,
    icon_inactive: MenuIcon,
    // Store last state to avoid redundant menu updates (prevents flickering/blurring)
    last_status: Mutex<String>,
    last_connection: Mutex<String>,
}

impl TrayManager {
    pub fn new() -> Result<Self> {
        // Load the main logo with high resolution for sharpness on High-DPI screens
        let wazuh_logo = load_tray_icon_from_assets("wazuh-logo.png")?;

        // Load shiny status dots (resized in Dockerfile to 22x22 for perfect sharpness)
        let (rgba_active, w_a, h_a) = load_icon_data("green-dot.png")?;
        let (rgba_inactive, w_i, h_i) = load_icon_data("gray-dot.png")?;

        let icon_active = MenuIcon::from_rgba(rgba_active, w_a, h_a)?;
        let icon_inactive = MenuIcon::from_rgba(rgba_inactive, w_i, h_i)?;

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Wazuh Agent Status")
            .with_icon(wazuh_logo)
            .build()
            .context("Error building tray icon")?;

        let manager = Self {
            tray_icon,
            icon_active,
            icon_inactive,
            last_status: Mutex::new(String::new()),
            last_connection: Mutex::new(String::new()),
        };

        // Initialize with default state
        manager.update_status("Unknown".to_string(), "Unknown".to_string());

        Ok(manager)
    }

    pub fn update_status(&self, status: String, connection: String) {
        {
            let mut last_s = self.last_status.lock().unwrap();
            let mut last_c = self.last_connection.lock().unwrap();

            if *last_s == status && *last_c == connection {
                return; // Prevent redundant updates and flickering
            }

            *last_s = status.clone();
            *last_c = connection.clone();
        }

        let is_active = status.to_lowercase() == "active";
        let is_connected = connection.to_lowercase() == "connected";

        let status_icon = if is_active { &self.icon_active } else { &self.icon_inactive };
        let conn_icon = if is_connected { &self.icon_active } else { &self.icon_inactive };

        let menu = Menu::new();
        
        let status_item = IconMenuItem::new(
            format!("Agent: {}", status),
            false, // Informative-only
            Some(status_icon.clone()),
            None,
        );
        
        let connection_item = IconMenuItem::new(
            format!("Connection: {}", connection),
            false,
            Some(conn_icon.clone()),
            None,
        );

        let version_item = MenuItem::with_id("version", "v0.1.0", false, None);

        let _ = menu.append(&status_item);
        let _ = menu.append(&connection_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&version_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&PredefinedMenuItem::quit(None));

        self.tray_icon.set_menu(Some(Box::new(menu)));
    }
}

fn load_tray_icon_from_assets(name: &str) -> Result<tray_icon::Icon> {
    let icon_data = Assets::get(name).context(format!("Asset {} missing", name))?;
    let img = image::load_from_memory(&icon_data.data).context(format!("Failed to decode {}", name))?;
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();
    tray_icon::Icon::from_rgba(rgba, width, height).context("Failed to generate RGBA tray icon")
}

fn load_icon_data(name: &str) -> Result<(Vec<u8>, u32, u32)> {
    let icon_data = Assets::get(name).context(format!("Asset {} missing", name))?;
    let img = image::load_from_memory(&icon_data.data).context(format!("Failed to decode {}", name))?;
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();
    Ok((rgba, width, height))
}
