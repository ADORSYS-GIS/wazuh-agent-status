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
pub struct EndpointsConfig {
    pub version_url: String,
    pub backend_port: u16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FeaturesConfig {
    pub self_healing: bool,
    pub log_streaming: bool,
    pub os_updates_check: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WazuhConfig {
    pub linux_base_path: String,
    pub macos_base_path: String,
    pub windows_base_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub brand: BrandConfig,
    pub endpoints: EndpointsConfig,
    pub features: FeaturesConfig,
    pub wazuh: WazuhConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrameworkVersion {
    pub version: String,
    pub prerelease_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionInfo {
    pub framework: FrameworkVersion,
    pub prerelease_test_groups: Option<Vec<String>>,
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
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client.get(&self.endpoints.version_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch version info: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Version check failed with status: {}", resp.status()));
        }

        let version_info: VersionInfo = resp.json()
            .await
            .map_err(|e| format!("Failed to parse version JSON: {}", e))?;

        let current_version_str = env!("CARGO_PKG_VERSION");
        let latest_version_str = &version_info.framework.version;

        use semver::Version;
        let current_v = Version::parse(current_version_str).map_err(|e| e.to_string())?;
        let latest_v = Version::parse(latest_version_str.trim_start_matches('v')).map_err(|e| e.to_string())?;

        let update_available = latest_v > current_v;

        Ok(UpdateCheckResult {
            current_version: current_version_str.to_string(),
            latest_version: latest_version_str.clone(),
            update_available,
            download_url: "https://github.com/ADORSYS-GIS/wazuh-agent/releases/latest".to_string(),
        })
    }

    pub async fn apply_update(&self, download_url: String) -> Result<(), String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client.get(download_url)
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Download failed with status: {}", resp.status()));
        }

        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        
        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let temp_exe = current_exe.with_extension("tmp");

        // Write new binary to a temporary file
        std::fs::write(&temp_exe, bytes).map_err(|e| format!("Failed to write temp file: {}", e))?;

        // Set permissions on Linux/macOS
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temp_exe, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }

        // Rename temp to current (hot-swap)
        // Note: On Windows this might fail if the file is locked, but on Linux it works great.
        std::fs::rename(&temp_exe, &current_exe)
            .map_err(|e| format!("Failed to replace executable: {}. You might need to run with higher permissions.", e))?;

        Ok(())
    }
    
    pub fn load(app: &tauri::AppHandle) -> Result<Self, String> {
        // ... (previous load code)
        use tauri::Manager;
        
        let config_path = app.path().resolve("app_config.json", tauri::path::BaseDirectory::AppConfig)
            .map_err(|e| format!("Failed to resolve config path: {}", e))?;

        if !config_path.exists() {
            // Fallback for development if not in AppConfigDir yet
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
