use crate::ai;
use crate::ai::client::{AiClient, AiModel, AiProviderConfig, AiProviderStatus};
use crate::ai::fixer::{AiFixResult, FailedCheckInput};

#[tauri::command]
pub async fn save_ai_config(config: AiProviderConfig) -> Result<(), String> {
    log::info!("Saving AI provider config for model: {}", config.model);
    ai::keychain::store_config(&config)
}

#[tauri::command]
pub async fn get_ai_status() -> AiProviderStatus {
    ai::keychain::get_provider_status()
}

#[tauri::command]
pub async fn get_ai_config() -> Result<AiProviderConfig, String> {
    ai::keychain::get_config()
}

#[tauri::command]
pub async fn test_ai_connection(config: AiProviderConfig) -> Result<String, String> {
    log::info!("Testing AI connection to: {}", config.base_url);
    let client = AiClient::new(config)?;
    client.ping().await
}

#[tauri::command]
pub async fn clear_ai_config() -> Result<(), String> {
    log::info!("Clearing AI provider config");
    ai::keychain::clear_config()
}

#[tauri::command]
pub async fn list_ai_models(config: AiProviderConfig) -> Result<Vec<AiModel>, String> {
    log::info!("Fetching models from: {}", config.base_url);
    let client = AiClient::new(config)?;
    client.list_models().await
}

#[tauri::command]
pub async fn ai_fix_check(input: FailedCheckInput) -> AiFixResult {
    ai::fixer::generate_fix(input).await
}

#[tauri::command]
pub async fn ai_fix_batch(inputs: Vec<FailedCheckInput>) -> Vec<AiFixResult> {
    ai::fixer::generate_fixes_batch(inputs).await
}

#[tauri::command]
pub async fn ai_chat(prompt: String, context: Option<String>) -> Result<String, String> {
    log::info!("AI chat: prompt length {}", prompt.len());

    let config = ai::keychain::get_config()?;
    let client = AiClient::new(config)?;

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

#[tauri::command]
pub async fn execute_fix_command(command: String) -> Result<String, String> {
    log::info!("AI Command Execution: '{}'", command);
    validate_command(&command)?;
    run_shell_command(&command)
}

#[tauri::command]
pub async fn execute_fix_command_sudo(command: String) -> Result<String, String> {
    log::info!("AI sudo Command Execution (native elevation)");
    validate_command(&command)?;

    let stripped = command
        .trim()
        .strip_prefix("sudo")
        .unwrap_or(&command)
        .trim()
        .to_string();

    run_elevated_command(&stripped)
}

#[tauri::command]
pub async fn trigger_sca_rescan() -> Result<String, String> {
    log::info!("Triggering SCA rescan via wazuh-agent restart");

    let cmd = if cfg!(target_os = "windows") {
        "powershell -NoProfile -Command \"Restart-Service -Name WazuhSvc -Force\" && echo Wazuh agent restarted successfully. SCA rescan started."
    } else if cfg!(target_os = "macos") {
        "/Library/Ossec/bin/wazuh-control restart && echo 'Wazuh agent restarted successfully. SCA rescan started.'"
    } else {
        "systemctl restart wazuh-agent && echo 'Wazuh agent restarted successfully. SCA rescan started.'"
    };

    run_elevated_command(cmd)
}

const ALLOWED_COMMANDS: &[&str] = &[
    // Unix core
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
    "chmod",
    "chown",
    "chgrp",
    "cp",
    "mv",
    "ln",
    "mkdir",
    "rmdir",
    "touch",
    "ls",
    "stat",
    "find",
    "test",
    "which",
    "type",
    "date",
    "sleep",
    "true",
    "false",
    "command",
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
    // User/group management
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
    // Crypto
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
    "visudo",
    // Wazuh
    "wazuh-agent",
    "wazuh-control",
    "ossec-control",
    // Shutdown
    "reboot",
    "shutdown",
    "poweroff",
    "halt",
    // macOS
    "xprotect",
    "pwpolicy",
    // PowerShell / Windows cmdlets
    "powershell",
    "powershell.exe",
    "cmd",
    "cmd.exe",
    "Get-Service",
    "Start-Service",
    "Stop-Service",
    "Restart-Service",
    "Set-Service",
    "New-Service",
    "Remove-Service",
    "Get-Process",
    "Stop-Process",
    "Start-Process",
    "Get-WmiObject",
    "Get-CimInstance",
    "Set-ExecutionPolicy",
    "Get-ExecutionPolicy",
    "Get-ItemProperty",
    "Set-ItemProperty",
    "Remove-ItemProperty",
    "New-ItemProperty",
    "New-Item",
    "Remove-Item",
    "Set-Item",
    "Get-Item",
    "New-LocalUser",
    "Remove-LocalUser",
    "Set-LocalUser",
    "Get-LocalUser",
    "Add-LocalGroupMember",
    "Remove-LocalGroupMember",
    "Get-LocalGroupMember",
    "Get-LocalGroup",
    "New-LocalGroup",
    "Remove-LocalGroup",
    "Enable-WindowsOptionalFeature",
    "Disable-WindowsOptionalFeature",
    "Get-WindowsOptionalFeature",
    "Set-NetFirewallRule",
    "New-NetFirewallRule",
    "Remove-NetFirewallRule",
    "Get-NetFirewallRule",
    "Set-NetIPAddress",
    "New-NetIPAddress",
    "Remove-NetIPAddress",
    "Get-NetIPAddress",
    "Set-NetAdapter",
    "Disable-NetAdapter",
    "Enable-NetAdapter",
    "Get-NetAdapter",
    "Set-NetConnectionProfile",
    "Get-NetConnectionProfile",
    "Install-WindowsFeature",
    "Uninstall-WindowsFeature",
    "Add-WindowsFeature",
    "Remove-WindowsFeature",
    "Get-WindowsFeature",
    "repadmin",
    "gpupdate",
    "gpresult",
    "secedit",
    "Where-Object",
    "Select-Object",
    "ForEach-Object",
    "Write-Host",
    "Write-Output",
    "Write-Error",
    "New-Object",
    // Windows system tools via cmd
    "Reg",
    "reg",
    "sc",
    "sc.exe",
    "icacls",
    "takeown",
    "net",
    "netstat",
    "taskkill",
    "tasklist",
];

fn normalize_separators(s: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' && !in_double {
            in_single = !in_single;
            out.push(c);
        } else if c == '"' && !in_single {
            in_double = !in_double;
            out.push(c);
        } else if !in_single && !in_double {
            if c == ';' {
                out.push_str(" ; ");
            } else if c == '&' && i + 1 < chars.len() && chars[i + 1] == '&' {
                out.push_str(" && ");
                i += 1;
            } else if c == '|' && i + 1 < chars.len() && chars[i + 1] == '|' {
                out.push_str(" || ");
                i += 1;
            } else if c == '|' {
                out.push_str(" | ");
            } else {
                out.push(c);
            }
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

fn extract_base_commands(command: &str) -> Vec<String> {
    let normalized = normalize_separators(command);
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let mut commands: Vec<String> = Vec::new();
    let mut expect_command = true;

    for token in &tokens {
        if expect_command {
            if *token == "sudo" || *token == "!" {
                continue;
            }
            if matches!(
                *token,
                "|" | ";" | "&" | "&&" | "||" | "<" | ">" | ">>" | "<<"
            ) {
                continue;
            }
            commands.push(token.to_string());
            expect_command = false;
        }

        if matches!(*token, "|" | ";" | "&&" | "||") {
            expect_command = true;
        }
    }

    commands
}

fn validate_find_exec(trimmed: &str) -> Result<(), String> {
    let normalized = normalize_separators(trimmed);
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];
        if (token == "-exec" || token == "-execdir")
            && let Some(target) = tokens.get(i + 1)
            && !target.starts_with('-')
            && !ALLOWED_COMMANDS.contains(target)
        {
            return Err(format!(
                "'{}' is not allowed as a target of `find -exec`",
                target
            ));
        }
        i += 1;
    }

    Ok(())
}

fn validate_command(command: &str) -> Result<(), String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err("Cannot execute an empty command".to_string());
    }

    if trimmed.contains('\n') {
        return Err("Multi-line commands are not allowed".to_string());
    }

    if trimmed.contains("$(") {
        return Err("Command contains shell substitution which is not allowed".to_string());
    }

    if trimmed.contains('`') {
        return Err("Command contains backtick substitution which is not allowed".to_string());
    }

    let base_commands = extract_base_commands(trimmed);
    if base_commands.is_empty() {
        return Err("Could not determine any command to execute".to_string());
    }

    for cmd in &base_commands {
        if !ALLOWED_COMMANDS.contains(&cmd.as_str()) {
            return Err(format!("'{}' is not in the allowed command list", cmd));
        }
    }

    if base_commands.iter().any(|c| c == "find") {
        validate_find_exec(trimmed)?;
    }

    Ok(())
}

fn parse_command_output(output: std::process::Output) -> Result<String, String> {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        if stdout.is_empty() && !stderr.is_empty() {
            Ok(stderr)
        } else if stdout.is_empty() {
            Ok("Command executed successfully".to_string())
        } else {
            Ok(stdout)
        }
    } else {
        Err(if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("Command exited with status: {:?}", output.status.code())
        })
    }
}

fn run_elevated_command(command: &str) -> Result<String, String> {
    if cfg!(target_os = "windows") {
        let random_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temp_file = std::env::temp_dir().join(format!("wazuh_elevated_out_{}.txt", random_id));
        let temp_file_str = temp_file.to_string_lossy().to_string();

        let escaped = command
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace("`", "``")
            .replace("$(", "`$(");
        let start_cmd = format!(
            "Start-Process powershell -ArgumentList '-NoProfile -NonInteractive -Command \"\"{} | Out-File -FilePath \\\"{}\\\" -Encoding UTF8; if (-not $?) {{ exit 1 }}\"\"' -Verb RunAs -Wait -WindowStyle Hidden",
            escaped, temp_file_str
        );

        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &start_cmd])
            .output();

        match output {
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    let _ = std::fs::remove_file(&temp_file);
                    return Err(format!("Failed to prompt elevation: {}", stderr));
                }
                if temp_file.exists() {
                    let content = std::fs::read_to_string(&temp_file)
                        .map_err(|e| format!("Failed to read output file: {}", e))?;
                    let _ = std::fs::remove_file(&temp_file);
                    Ok(content)
                } else {
                    Err(
                        "Elevation was cancelled or command output could not be captured"
                            .to_string(),
                    )
                }
            }
            Err(e) => {
                let _ = std::fs::remove_file(&temp_file);
                Err(format!("System failed to execute elevated process: {}", e))
            }
        }
    } else if cfg!(target_os = "macos") {
        let escaped_cmd = command.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "do shell script \"{}\" with administrator privileges",
            escaped_cmd
        );
        let output = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("System failed to execute elevated process: {}", e))?;
        parse_command_output(output)
    } else {
        let output = std::process::Command::new("pkexec")
            .args(["sh", "-c", command])
            .output()
            .map_err(|e| format!("System failed to execute elevated process: {}", e))?;
        parse_command_output(output)
    }
}

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
        Ok(out) => parse_command_output(out),
        Err(err) => {
            log::error!("Failed to execute command: {:?}", err);
            Err(format!("System failed to execute process: {}", err))
        }
    }
}
