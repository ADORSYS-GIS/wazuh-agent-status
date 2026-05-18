use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    pub primary_color: String,
    pub secondary_color: String,
    pub dark_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrandConfig {
    pub name: String,
    pub company: String,
    pub logo_path: String,
    pub theme: ThemeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeaturesConfig {
    pub self_healing: bool,
    pub log_streaming: bool,
    pub os_updates_check: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub server_addr: String,
    pub brand: BrandConfig,
    pub features: FeaturesConfig,
}

impl AppConfig {
    pub fn load(app: &tauri::AppHandle) -> Result<Self, String> {
        use tauri::Manager;

        // 1. Try bundled resource (allows override in .deb/.msi/.app installs)
        match app.path().resolve("app_config.json", tauri::path::BaseDirectory::Resource) {
            Ok(resource_path) if resource_path.exists() => {
                return Self::load_from_path(resource_path);
            }
            Ok(_) => log::info!("Bundled app_config.json not found in resources."),
            Err(e) => log::debug!("Could not resolve resource path (expected when not bundled): {}", e),
        }

        // 2. Fallback: current working directory (for dev / cargo run / npm run tauri dev)
        let dev_path = std::path::PathBuf::from("app_config.json");
        if dev_path.exists() {
            return Self::load_from_path(dev_path);
        }
        log::info!("app_config.json not found in working directory.");

        // 3. Ultimate fallback: compile-time embedded config
        //    Guaranteed to exist for raw binaries moved anywhere (e.g. /usr/local/bin)
        log::info!("Using embedded application configuration.");
        let embedded = include_str!("../app_config.json");
        serde_json::from_str(embedded)
            .map_err(|e| format!("Failed to parse embedded config: {}", e))
    }

    fn load_from_path(path: std::path::PathBuf) -> Result<Self, String> {
        let config_str = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file at {:?}: {}", path, e))?;

        let config: AppConfig = serde_json::from_str(&config_str)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }
}
