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
    pub brand: BrandConfig,
    pub features: FeaturesConfig,
}

#[derive(Debug, Serialize, Clone)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: String,
}

impl AppConfig {
    pub async fn check_for_updates(&self) -> Result<UpdateCheckResult, String> {
        Ok(UpdateCheckResult {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            latest_version: env!("CARGO_PKG_VERSION").to_string(),
            update_available: false,
            download_url: "".to_string(),
        })
    }

    pub async fn apply_update(&self, _download_url: String) -> Result<(), String> {
        Ok(())
    }

    pub fn load(app: &tauri::AppHandle) -> Result<Self, String> {
        use tauri::Manager;

        let config_path = app
            .path()
            .resolve("app_config.json", tauri::path::BaseDirectory::AppConfig)
            .map_err(|e| format!("Failed to resolve config path: {}", e))?;

        if !config_path.exists() {
            let dev_path = std::path::PathBuf::from("app_config.json");
            if dev_path.exists() {
                return Self::load_from_path(dev_path);
            }
            return Err(format!("Config file not found at {:?}", config_path));
        }

        Self::load_from_path(config_path)
    }

    fn load_from_path(path: std::path::PathBuf) -> Result<Self, String> {
        let config_str = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file at {:?}: {}", path, e))?;

        let config: AppConfig = serde_json::from_str(&config_str)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }
}
