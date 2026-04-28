//! Application configuration and platform-specific path resolution.
//!
//! [`Config`] is built from environment variables (with safe defaults).
//! [`AgentPaths`] centralises every file-system path used to interrogate
//! the Wazuh agent installation, resolved once at start-up for the current OS.

use std::path::PathBuf;
use std::time::Duration;
use tracing::warn;

// ── Defaults ─────────────────────────────────────────────────────────────────

const DEFAULT_VERSION_URL: &str =
    "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/heads/main/versions.json";
const DEFAULT_POLL_INTERVAL_SECS: u64 = 5;
const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:50505";
/// Cache remote version checks for 30 minutes to avoid hammering GitHub.
const DEFAULT_VERSION_CACHE_TTL_SECS: u64 = 1_800;

// ── Config ───────────────────────────────────────────────────────────────────

/// Server runtime configuration.
///
/// All fields can be overridden at start-up via environment variables:
///
/// | Field               | Env var                                  | Default            |
/// |---------------------|------------------------------------------|--------------------|
/// | `listen_addr`       | `WAZUH_STATUS_ADDR`                      | `0.0.0.0:50505`    |
/// | `log_file`          | `WAZUH_STATUS_LOG_FILE`                  | `/var/log/...`     |
/// | `poll_interval`     | `WAZUH_STATUS_POLL_INTERVAL_SECS`        | `5`                |
/// | `version_url`       | `WAZUH_STATUS_VERSION_URL`               | GitHub manifest    |
/// | `version_cache_ttl` | `WAZUH_STATUS_VERSION_CACHE_TTL_SECS`    | `1800`             |
#[derive(Debug, Clone)]
pub struct Config {
    /// TCP socket address the server will listen on.
    pub listen_addr: String,
    /// How often the background task polls the local Wazuh agent state.
    pub poll_interval: Duration,
    /// URL of the remote version manifest JSON.
    pub version_url: String,
    /// How long a remote version check result is cached before re-fetching.
    pub version_cache_ttl: Duration,
    /// Maximum concurrent client connections allowed.
    pub max_connections: usize,
    /// Whether to automatically restart the Wazuh agent if it is inactive.
    pub self_healing: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr:       DEFAULT_LISTEN_ADDR.to_string(),
            poll_interval:     Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS),
            version_url:       DEFAULT_VERSION_URL.to_string(),
            version_cache_ttl: Duration::from_secs(DEFAULT_VERSION_CACHE_TTL_SECS),
            max_connections:   3,
            self_healing:      true,
        }
    }
}

impl Config {
    /// Construct a [`Config`] from environment variables, falling back to [`Default`] values.
    pub fn from_env() -> Self {
        let mut cfg = Self::default();

        if let Ok(addr) = std::env::var("WAZUH_STATUS_ADDR") {
            // Security check: Warn if binding to non-localhost (as there's no auth)
            if !addr.contains("localhost") && !addr.contains("127.0.0.1") && !addr.contains("[::1]") {
                warn!(
                    addr = %addr, 
                    "WARNING: Server is binding to a public interface without authentication. Ensure it is protected by a firewall."
                );
            }
            cfg.listen_addr = addr;
        }

        if let Ok(raw) = std::env::var("WAZUH_STATUS_POLL_INTERVAL_SECS") {
            match raw.parse::<u64>() {
                Ok(secs) => cfg.poll_interval = Duration::from_secs(secs),
                Err(_) => warn!(
                    env_var = "WAZUH_STATUS_POLL_INTERVAL_SECS",
                    value = %raw,
                    "Invalid value; using default"
                ),
            }
        }

        if let Ok(url) = std::env::var("WAZUH_STATUS_VERSION_URL") {
            cfg.version_url = url;
        }

        if let Ok(raw) = std::env::var("WAZUH_STATUS_VERSION_CACHE_TTL_SECS") {
            match raw.parse::<u64>() {
                Ok(secs) => cfg.version_cache_ttl = Duration::from_secs(secs),
                Err(_) => warn!(
                    env_var = "WAZUH_STATUS_VERSION_CACHE_TTL_SECS",
                    value = %raw,
                    "Invalid value; using default"
                ),
            }
        }

        if let Ok(raw) = std::env::var("WAZUH_STATUS_MAX_CONNECTIONS") {
            match raw.parse::<usize>() {
                Ok(n) => cfg.max_connections = n,
                Err(_) => warn!(
                    env_var = "WAZUH_STATUS_MAX_CONNECTIONS",
                    value = %raw,
                    "Invalid value; using default"
                ),
            }
        }

        if let Ok(raw) = std::env::var("WAZUH_STATUS_SELF_HEALING") {
            match raw.to_lowercase().as_str() {
                "true" | "1" | "yes" => cfg.self_healing = true,
                "false" | "0" | "no" => cfg.self_healing = false,
                _ => warn!(
                    env_var = "WAZUH_STATUS_SELF_HEALING",
                    value = %raw,
                    "Invalid value; using default"
                ),
            }
        }

        cfg
    }
}

// ── AgentPaths ───────────────────────────────────────────────────────────────

/// Resolved file-system paths for the Wazuh agent installation on the current OS.
///
/// Create with [`AgentPaths::native()`] to get the correct paths for the
/// platform this binary was compiled for.
#[derive(Debug, Clone)]
pub struct AgentPaths {
    /// Agent state file — used to determine the connection status.
    pub state_file: PathBuf,
    /// Installed agent version file (JSON).
    pub version_json: PathBuf,
    /// Tray application version file.
    pub version_file: PathBuf,
    /// Group policy merged configuration file.
    pub merged_mg: PathBuf,
    /// Daemon PID file (UNIX only; empty on Windows).
    pub pid_file: PathBuf,
    /// Path to the update wrapper script.
    pub update_script: PathBuf,
    /// Path to the wazuh-control utility.
    pub wazuh_control: PathBuf,
    /// Path to the Wazuh agent's ossec.log file.
    pub ossec_log: PathBuf,
}

impl AgentPaths {
    /// Return the native paths for the OS this binary targets.
    pub fn native() -> Self {
        let ossec_log_override = std::env::var("WAZUH_STATUS_OSSEC_LOG").ok().map(PathBuf::from);

        #[cfg(target_os = "linux")]
        {
            let base = PathBuf::from("/var/ossec");
            Self {
                state_file:    base.join("var/run/wazuh-agentd.state"),
                version_file:  base.join("etc/version.txt"),
                version_json:  base.join("VERSION.json"),
                merged_mg:     base.join("etc/shared/merged.mg"),
                pid_file:      base.join("var/run/wazuh-agentd.pid"),
                update_script: base.join("active-response/bin/adorsys-update.sh"),
                wazuh_control: base.join("bin/wazuh-control"),
                ossec_log:     ossec_log_override.unwrap_or_else(|| base.join("logs/ossec.log")),
            }
        }

        #[cfg(target_os = "macos")]
        {
            let base = PathBuf::from("/Library/Ossec");
            Self {
                state_file:    base.join("var/run/wazuh-agentd.state"),
                version_file:  base.join("etc/version.txt"),
                version_json:  base.join("VERSION.json"),
                merged_mg:     base.join("etc/shared/merged.mg"),
                pid_file:      base.join("var/run/wazuh-agentd.pid"),
                update_script: base.join("active-response/bin/adorsys-update.sh"),
                wazuh_control: base.join("bin/wazuh-control"),
                ossec_log:     ossec_log_override.unwrap_or_else(|| base.join("logs/ossec.log")),
            }
        }

        #[cfg(target_os = "windows")]
        {
            let base = PathBuf::from(r"C:\Program Files (x86)\ossec-agent");
            Self {
                state_file:    base.join("wazuh-agent.state"),
                version_file:  base.join("version.txt"),
                version_json:  base.join("VERSION.json"),
                merged_mg:     base.join(r"shared\merged.mg"),
                pid_file:      PathBuf::new(), // not applicable on Windows
                update_script: base.join("adorsys-update.bat"),
                wazuh_control: base.join("wazuh-control.exe"), // Placeholder for Windows
                ossec_log:     ossec_log_override.unwrap_or_else(|| base.join(r"logs\ossec.log")),
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            panic!("Unsupported platform — no AgentPaths defined for this OS")
        }
    }

    /// Platform-specific path to the server's own log file.
    ///
    /// Exposed as a stand-alone method so the logging system can be
    /// initialised before a full [`AgentPaths`] instance is built.
    pub fn log_file_path() -> PathBuf {
        // Allow override via environment variable
        if let Ok(val) = std::env::var("WAZUH_STATUS_LOG_FILE") {
            return PathBuf::from(val);
        }

        #[cfg(target_os = "windows")]
        return PathBuf::from(r"C:\ProgramData\wazuh\logs\wazuh-agent-status.log");

        // Linux and macOS share the same path
        #[cfg(not(target_os = "windows"))]
        PathBuf::from("/var/log/wazuh-agent-status.log")
    }
}
