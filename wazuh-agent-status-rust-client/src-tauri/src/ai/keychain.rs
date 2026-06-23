//! File-based AI provider credential storage.
//!
//! The config (base URL, model, API key) is stored in
//! `~/.config/wazuh-agent-status/ai-config.json` with `0o600` permissions.
//! No OS keychain dependency needed — works reliably on all platforms.

use crate::ai::client::AiProviderConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Full config stored on disk (key included, file permissions restricted).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

// ── Path helpers ──────────────────────────────────────────────────────────────

fn config_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/wazuh-agent-status")
}

fn config_file() -> PathBuf {
    config_dir().join("ai-config.json")
}

// ── File read/write ───────────────────────────────────────────────────────────

fn write_config(cfg: &FileConfig) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config directory: {e}"))?;

    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;

    let path = config_file();
    fs::write(&path, &json).map_err(|e| format!("Failed to write config file: {e}"))?;

    // Restrict permissions (owner read/write only) on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(&path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(&path, perms);
        }
    }

    Ok(())
}

fn read_config_raw() -> Result<FileConfig, String> {
    let path = config_file();
    let json = fs::read_to_string(&path).map_err(|_| "No AI provider configured".to_string())?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse config file: {e}"))
}

fn remove_config_file() {
    let _ = fs::remove_file(config_file());
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Store a full [`AiProviderConfig`] to the config file.
pub fn store_config(config: &AiProviderConfig) -> Result<(), String> {
    let fc = FileConfig {
        base_url: config.base_url.clone(),
        model: config.model.clone(),
        api_key: config.api_key.clone(),
    };
    write_config(&fc)?;
    log::info!("AI config saved to {}", config_file().display());
    Ok(())
}

/// Read the full [`AiProviderConfig`] from the config file.
pub fn get_config() -> Result<AiProviderConfig, String> {
    let fc = read_config_raw()?;
    Ok(AiProviderConfig {
        api_key: fc.api_key,
        base_url: fc.base_url,
        model: fc.model,
        ..Default::default()
    })
}

/// Return provider status (safe for frontend — no API key).
///
/// Never returns an error: if no config is found, returns
/// `{ configured: false }`.
pub fn get_provider_status() -> super::client::AiProviderStatus {
    match get_config() {
        Ok(cfg) => super::client::AiProviderStatus {
            base_url: cfg.base_url,
            model: cfg.model,
            configured: true,
        },
        Err(_) => super::client::AiProviderStatus {
            base_url: String::new(),
            model: String::new(),
            configured: false,
        },
    }
}

/// Remove the config file.
pub fn clear_config() -> Result<(), String> {
    remove_config_file();
    log::info!("AI config cleared");
    Ok(())
}
