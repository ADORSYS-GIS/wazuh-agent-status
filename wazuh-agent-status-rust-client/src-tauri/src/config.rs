use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    pub primary_color: String,
    pub secondary_color: String,
    pub dark_mode: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidebar_bg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_dim: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrandConfig {
    pub name: String,
    pub company: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_by: Option<String>,
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
    pub gateway_url: String,
    pub brand: BrandConfig,
    pub features: FeaturesConfig,
}

impl AppConfig {
    pub fn load(app: &tauri::AppHandle) -> Result<Self, String> {
        use tauri::Manager;

        // 1. Try bundled resource (allows override in .deb/.msi/.app installs)
        let mut config = match app
            .path()
            .resolve("app_config.json", tauri::path::BaseDirectory::Resource)
        {
            Ok(resource_path) if resource_path.exists() => {
                Some(Self::load_from_path(resource_path)?)
            }
            Ok(_) => {
                log::info!("Bundled app_config.json not found in resources.");
                None
            }
            Err(e) => {
                log::debug!(
                    "Could not resolve resource path (expected when not bundled): {}",
                    e
                );
                None
            }
        };

        // 2. Fallback: current working directory (for dev / cargo run / npm run tauri dev)
        if config.is_none() {
            let dev_path = std::path::PathBuf::from("app_config.json");
            if dev_path.exists() {
                config = Some(Self::load_from_path(dev_path)?);
            } else {
                log::info!("app_config.json not found in working directory.");
            }
        }

        // 3. Ultimate fallback: compile-time embedded config
        let config = match config {
            Some(c) => c,
            None => {
                log::info!("Using embedded application configuration.");
                let embedded = include_str!("../app_config.json");
                serde_json::from_str(embedded)
                    .map_err(|e| format!("Failed to parse embedded config: {}", e))?
            }
        };

        // 4. Environment variable override for gateway URL
        if let Ok(url) = std::env::var("WAZUH_GATEWAY_URL") {
            log::info!(
                "Overriding gateway_url from WAZUH_GATEWAY_URL env var: {}",
                url
            );
            Ok(AppConfig {
                gateway_url: url,
                ..config
            })
        } else {
            Ok(config)
        }
    }

    fn load_from_path(path: std::path::PathBuf) -> Result<Self, String> {
        let config_str = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config file at {:?}: {}", path, e))?;

        let config: AppConfig = serde_json::from_str(&config_str)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_path_valid() {
        let temp_path = std::env::temp_dir().join("test_app_config.json");
        let json = r##"{
            "server_addr": "127.0.0.1:9099",
            "gateway_url": "https://gateway.example.com",
            "brand": {
                "name": "Wazuh Agent",
                "company": "Adorsys",
                "logo_path": "/icons/logo.png",
                "theme": {
                    "primary_color": "#0055ff",
                    "secondary_color": "#ffffff",
                    "dark_mode": true
                }
            },
            "features": {
                "self_healing": true,
                "log_streaming": true,
                "os_updates_check": false
            }
        }"##;
        std::fs::write(&temp_path, json).unwrap();

        let cfg = AppConfig::load_from_path(temp_path.clone()).unwrap();
        assert_eq!(cfg.server_addr, "127.0.0.1:9099");
        assert_eq!(cfg.gateway_url, "https://gateway.example.com");
        assert_eq!(cfg.brand.name, "Wazuh Agent");
        assert!(cfg.brand.theme.dark_mode);
        assert!(cfg.features.self_healing);
        assert!(!cfg.features.os_updates_check);

        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_load_from_path_missing_and_invalid() {
        let missing = std::path::PathBuf::from("/non/existent/path/config.json");
        assert!(AppConfig::load_from_path(missing).is_err());

        let invalid_path = std::env::temp_dir().join("test_invalid_app_config.json");
        std::fs::write(&invalid_path, b"not json").unwrap();
        assert!(AppConfig::load_from_path(invalid_path.clone()).is_err());
        let _ = std::fs::remove_file(invalid_path);
    }

    #[test]
    fn test_embedded_app_config_validity() {
        let embedded = include_str!("../app_config.json");
        let parsed: AppConfig = serde_json::from_str(embedded).unwrap();
        assert!(!parsed.server_addr.is_empty());
        assert!(!parsed.gateway_url.is_empty());
        assert!(!parsed.brand.name.is_empty());
        assert!(!parsed.brand.theme.primary_color.is_empty());
    }
}
