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

fn config_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| {
            "Cannot determine home directory: neither HOME nor USERPROFILE is set. \
             Refusing to store API key in an unknown location."
                .to_string()
        })?;
    Ok(PathBuf::from(home).join(".config/wazuh-agent-status"))
}

fn config_file() -> Result<PathBuf, String> {
    Ok(config_dir()?.join("ai-config.json"))
}

// ── File read/write ───────────────────────────────────────────────────────────

fn write_config(cfg: &FileConfig) -> Result<(), String> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config directory: {e}"))?;

    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;

    let path = config_file()?;
    fs::write(&path, &json).map_err(|e| format!("Failed to write config file: {e}"))?;

    // Restrict permissions (owner read/write only) on Unix.
    // Failure is treated as a hard error: we must not leave the API key
    // world-readable if the permission change cannot be applied.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta =
            fs::metadata(&path).map_err(|e| format!("Failed to read config file metadata: {e}"))?;
        let mut perms = meta.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)
            .map_err(|e| format!("Failed to restrict config file permissions: {e}"))?;
    }

    Ok(())
}

fn read_config_raw() -> Result<FileConfig, String> {
    let path = config_file()?;
    let json = fs::read_to_string(&path).map_err(|_| "No AI provider configured".to_string())?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse config file: {e}"))
}

fn remove_config_file() {
    // Best-effort removal: silently ignore errors (file may not exist).
    if let Ok(path) = config_file() {
        let _ = fs::remove_file(path);
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Store a full [`AiProviderConfig`] to the config file.
///
/// If `api_key` is empty, the existing API key from the config file is
/// preserved — this allows the frontend to update base URL / model
/// without requiring the user to re-enter the key every time.
pub fn store_config(config: &AiProviderConfig) -> Result<(), String> {
    let api_key = if config.api_key.is_empty() {
        // Preserve existing API key when not explicitly provided
        match read_config_raw() {
            Ok(existing) => existing.api_key,
            Err(_) => config.api_key.clone(),
        }
    } else {
        config.api_key.clone()
    };

    let fc = FileConfig {
        base_url: config.base_url.clone(),
        model: config.model.clone(),
        api_key,
    };
    write_config(&fc)?;
    if let Ok(path) = config_file() {
        log::info!("AI config saved to {}", path.display());
    }
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
            configured: !cfg.api_key.is_empty(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;

    /// Serialize filesystem-based tests to avoid `HOME` / temp-dir collisions.
    static FS_LOCK: Mutex<()> = Mutex::new(());

    /// Create a temporary `$HOME`, run `f`, then restore the original.
    fn with_temp_home<F>(f: F)
    where
        F: FnOnce(&Path),
    {
        let _lock = FS_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let tmp = std::env::temp_dir().join(format!("ai_keychain_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("failed to create temp dir");

        let prev = std::env::var("HOME").ok();
        // SAFETY: tests are serialized via FS_LOCK, so concurrent env var
        // modification cannot happen. HOME is restored before the lock is
        // released.
        unsafe { std::env::set_var("HOME", &tmp) };

        f(&tmp);

        match prev {
            Some(v) => unsafe { std::env::set_var("HOME", v) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        let _ = fs::remove_dir_all(&tmp);
    }

    // ── store_config persistence tests ────────────────────────────────────

    #[test]
    fn test_store_config_preserves_api_key_when_empty() {
        with_temp_home(|_home| {
            // 1. Save a config WITH an API key.
            store_config(&AiProviderConfig {
                base_url: "https://api.openai.com/v1".into(),
                model: "gpt-4o".into(),
                api_key: "sk-test-123".into(),
                ..Default::default()
            })
            .expect("first store should succeed");

            // 2. Save a config with EMPTY api_key (e.g. user only changed URL/model).
            store_config(&AiProviderConfig {
                base_url: "https://custom-proxy.example.com/v1".into(),
                model: "gpt-4o-mini".into(),
                api_key: String::new(),
                ..Default::default()
            })
            .expect("second store should succeed");

            // 3. Read back — api_key should be PRESERVED from step 1.
            let result = get_config().expect("get_config should succeed");
            assert_eq!(result.base_url, "https://custom-proxy.example.com/v1");
            assert_eq!(result.model, "gpt-4o-mini");
            assert_eq!(
                result.api_key, "sk-test-123",
                "API key should be preserved when not explicitly provided"
            );
        });
    }

    #[test]
    fn test_store_config_overwrites_api_key_when_provided() {
        with_temp_home(|_home| {
            // 1. Save with one API key.
            store_config(&AiProviderConfig {
                api_key: "sk-old-key".into(),
                ..Default::default()
            })
            .expect("first store should succeed");

            // 2. Save with a DIFFERENT (non-empty) api_key.
            store_config(&AiProviderConfig {
                api_key: "sk-new-key".into(),
                ..Default::default()
            })
            .expect("second store should succeed");

            // 3. Read back — api_key should be the NEW value.
            let result = get_config().expect("get_config should succeed");
            assert_eq!(
                result.api_key, "sk-new-key",
                "API key should be overwritten when a new key is provided"
            );
        });
    }

    #[test]
    fn test_store_config_empty_key_no_existing_file() {
        with_temp_home(|_home| {
            // No config file exists yet → save with empty api_key.
            store_config(&AiProviderConfig {
                base_url: "https://ollama.local/v1".into(),
                model: "llama3".into(),
                api_key: String::new(),
                ..Default::default()
            })
            .expect("store with empty key on fresh state should succeed");

            let result = get_config().expect("get_config should succeed");
            assert!(
                result.api_key.is_empty(),
                "API key should be empty when no existing file had a key"
            );
            assert_eq!(result.base_url, "https://ollama.local/v1");
            assert_eq!(result.model, "llama3");
        });
    }

    // ── clear_config ──────────────────────────────────────────────────────

    #[test]
    fn test_clear_config_removes_file() {
        with_temp_home(|_home| {
            store_config(&AiProviderConfig {
                api_key: "sk-keep".into(),
                ..Default::default()
            })
            .expect("store should succeed");

            assert!(get_config().is_ok(), "config should exist before clear");

            clear_config().expect("clear should succeed");
            assert!(get_config().is_err(), "config should not exist after clear");
        });
    }

    #[test]
    fn test_clear_config_when_no_file_exists() {
        with_temp_home(|_home| {
            // Should not panic or error when there's nothing to clear.
            clear_config().expect("clearing non-existent config should succeed");
        });
    }

    // ── get_provider_status ───────────────────────────────────────────────

    #[test]
    fn test_provider_status_configured() {
        with_temp_home(|_home| {
            store_config(&AiProviderConfig {
                base_url: "https://example.com/v1".into(),
                model: "test-model".into(),
                api_key: "sk-present".into(),
                ..Default::default()
            })
            .expect("store should succeed");

            let status = get_provider_status();
            assert!(status.configured);
            assert_eq!(status.base_url, "https://example.com/v1");
            assert_eq!(status.model, "test-model");
        });
    }

    #[test]
    fn test_provider_status_not_configured() {
        with_temp_home(|_home| {
            // No config file written → status should show not configured.
            let status = get_provider_status();
            assert!(!status.configured);
            assert!(status.base_url.is_empty());
            assert!(status.model.is_empty());
        });
    }
}
