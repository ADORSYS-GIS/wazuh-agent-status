//! Tauri commands for AI integration.
//!
//! # Exposed Commands
//!
//! | Command               | Purpose                                        |
//! |-----------------------|------------------------------------------------|
//! | `save_ai_config`      | Store provider config to file                  |
//! | `get_ai_status`       | Return provider status (safe for frontend)     |
//! | `test_ai_connection`  | Ping the AI provider to verify credentials     |
//! | `clear_ai_config`     | Remove stored credentials                      |
//! | `ai_fix_check`        | Generate an AI fix for a single failed SCA check|
//! | `ai_chat`             | Send a follow-up chat message to the AI        |

use crate::ai;
use crate::ai::client::{AiClient, AiModel, AiProviderConfig, AiProviderStatus};
use crate::ai::fixer::{AiFixResult, FailedCheckInput};

// ── Commands ──────────────────────────────────────────────────────────────────

/// Save the AI provider configuration to the config file.
#[tauri::command]
pub async fn save_ai_config(config: AiProviderConfig) -> Result<(), String> {
    log::info!("Saving AI provider config for model: {}", config.model);
    ai::keychain::store_config(&config)
}

/// Return the current AI provider status (safe for frontend display).
#[tauri::command]
pub async fn get_ai_status() -> AiProviderStatus {
    ai::keychain::get_provider_status()
}

/// Test the AI provider connection by sending a minimal ping request.
#[tauri::command]
pub async fn test_ai_connection(config: AiProviderConfig) -> Result<String, String> {
    log::info!("Testing AI connection to: {}", config.base_url);
    let client = AiClient::new(config)?;
    client.ping().await
}

/// Remove all stored AI credentials.
#[tauri::command]
pub async fn clear_ai_config() -> Result<(), String> {
    log::info!("Clearing AI provider config");
    ai::keychain::clear_config()
}

/// Fetch available models from the AI provider.
#[tauri::command]
pub async fn list_ai_models(config: AiProviderConfig) -> Result<Vec<AiModel>, String> {
    log::info!("Fetching models from: {}", config.base_url);
    let client = AiClient::new(config)?;
    client.list_models().await
}

/// Generate an AI-powered fix for a single failed SCA check.
#[tauri::command]
pub async fn ai_fix_check(input: FailedCheckInput) -> AiFixResult {
    ai::fixer::generate_fix(input).await
}

/// Generate AI-powered fixes for multiple failed SCA checks.
#[tauri::command]
pub async fn ai_fix_batch(inputs: Vec<FailedCheckInput>) -> Vec<AiFixResult> {
    ai::fixer::generate_fixes_batch(inputs).await
}

/// Send a follow-up chat message to the AI, with optional context from a prior fix.
///
/// `context` is the previous AI response markdown (so the model remembers what
/// was discussed). `prompt` is the user's follow-up question.
#[tauri::command]
pub async fn ai_chat(prompt: String, context: Option<String>) -> Result<String, String> {
    log::info!("AI chat: prompt length {}", prompt.len());

    let config = ai::keychain::get_config()?;
    let client = AiClient::new(config)?;

    // Build a contextual prompt
    let full_prompt = match &context {
        Some(ctx) if !ctx.is_empty() => {
            format!(
                "Previous context (from our last exchange):\n\n{ctx}\n\n---\n\nNow the user asks:\n{prompt}"
            )
        }
        _ => prompt.clone(),
    };

    client.chat(&full_prompt).await
}

/// Run a command suggested by the AI on the host system.
/// This variant handles commands that do NOT require sudo.
#[tauri::command]
pub async fn execute_fix_command(command: String) -> Result<String, String> {
    log::info!("AI Command Execution requested: '{}'", command);

    run_shell_command(&command)
}

/// Run a sudo command with the provided password, using `sudo -S` (stdin-based auth).
/// This avoids the TTY prompt that causes the app to hang.
#[tauri::command]
pub async fn execute_fix_command_sudo(
    command: String,
    sudo_password: String,
) -> Result<String, String> {
    log::info!("AI sudo Command Execution requested (password provided)");

    // Strip any leading "sudo" so we can reconstruct the pipeline ourselves
    let stripped = command
        .trim()
        .strip_prefix("sudo")
        .unwrap_or(&command)
        .trim()
        .to_string();
    let sudo_cmd = format!(
        "echo {pw} | sudo -S -p '' {cmd}",
        pw = shell_escape(&sudo_password),
        cmd = stripped
    );

    run_shell_command(&sudo_cmd)
}

/// Restart the local wazuh-agent service to force an immediate SCA rescan.
///
/// This is the only native Wazuh method to trigger an on-demand SCA scan.
/// Wazuh is configured with `<scan_on_start>yes</scan_on_start>` by default,
/// so a restart immediately runs a fresh scan and pushes results to the manager.
#[tauri::command]
pub async fn trigger_sca_rescan(sudo_password: String) -> Result<String, String> {
    log::info!("Triggering SCA rescan via wazuh-agent restart");

    let cmd = format!(
        "echo {pw} | sudo -S -p '' systemctl restart wazuh-agent && echo 'Wazuh agent restarted successfully. SCA rescan started.'",
        pw = shell_escape(&sudo_password)
    );

    run_shell_command(&cmd)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Shell-escape a string for safe interpolation into a shell command.
/// This wraps the value in single quotes and escapes any embedded single quotes.
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Run a shell command and return combined stdout or a descriptive error.
fn run_shell_command(command: &str) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", command])
            .output()
    } else {
        std::process::Command::new("sh")
            .args(["-c", command])
            .output()
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();

            if out.status.success() {
                if stdout.is_empty() && !stderr.is_empty() {
                    Ok(stderr)
                } else if stdout.is_empty() {
                    Ok("Command executed successfully (no output).".to_string())
                } else {
                    Ok(stdout)
                }
            } else {
                // Filter out the sudo password prompt echo if it leaked
                let err_msg = if !stderr.is_empty() {
                    stderr
                } else if !stdout.is_empty() {
                    stdout
                } else {
                    format!("Command exited with status code: {:?}", out.status.code())
                };
                Err(err_msg)
            }
        }
        Err(err) => {
            log::error!("Failed to execute command: {:?}", err);
            Err(format!("System failed to execute process: {}", err))
        }
    }
}
