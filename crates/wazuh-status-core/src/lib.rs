use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Active,
    Inactive,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Unknown,
}

#[cfg(not(windows))]
const WAZUH_CONTROL_PATH: &str = if cfg!(target_os = "macos") {
    "/Library/Ossec/bin/wazuh-control"
} else {
    "/var/ossec/bin/wazuh-control"
};

#[cfg(not(windows))]
const WAZUH_STATE_PATH: &str = if cfg!(target_os = "macos") {
    "/Library/Ossec/var/run/wazuh-agentd.state"
} else {
    "/var/ossec/var/run/wazuh-agentd.state"
};

#[cfg(not(windows))]
const UPDATE_SCRIPT_PATH: &str = if cfg!(target_os = "macos") {
    "/Library/Ossec/active-response/bin/adorsys-update.sh"
} else {
    "/var/ossec/active-response/bin/adorsys-update.sh"
};

#[cfg(windows)]
const POWERSHELL_EXE: &str = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe";

#[cfg(windows)]
const VERSION_FILE_PATH: &str = "C:\\Program Files (x86)\\ossec-agent\\version.txt";

#[cfg(not(windows))]
const VERSION_FILE_PATH: &str = if cfg!(target_os = "macos") {
    "/Library/Ossec/etc/version.txt"
} else {
    "/var/ossec/etc/version.txt"
};

pub fn check_service_status() -> Result<(AgentState, ConnectionState)> {
    #[cfg(windows)]
    {
        let output = Command::new(POWERSHELL_EXE)
            .args(["-Command", "Get-Service", "-Name", "WazuhSvc"])
            .output()
            .context("failed to query WazuhSvc status")?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !output.status.success() {
            return Ok((AgentState::Inactive, ConnectionState::Disconnected));
        }

        let agent_state = if stdout.contains("Running") {
            AgentState::Active
        } else {
            AgentState::Inactive
        };

        let conn_output = Command::new(POWERSHELL_EXE)
            .args([
                "-Command",
                "Select-String",
                "-Path",
                "C:\\Program Files (x86)\\ossec-agent\\wazuh-agent.state",
                "-Pattern",
                "^status",
            ])
            .output()
            .context("failed to query connection status")?;
        let conn_stdout = String::from_utf8_lossy(&conn_output.stdout);
        let connection_state = if conn_output.status.success()
            && conn_stdout.contains("status='connected'")
        {
            ConnectionState::Connected
        } else {
            ConnectionState::Disconnected
        };

        return Ok((agent_state, connection_state));
    }

    #[cfg(not(windows))]
    {
        let output = Command::new(WAZUH_CONTROL_PATH)
            .arg("status")
            .output()
            .context("failed to run wazuh-control status")?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let agent_state = if stdout.contains("wazuh-agentd is running") {
            AgentState::Active
        } else {
            AgentState::Inactive
        };

        let state_contents = std::fs::read_to_string(WAZUH_STATE_PATH).unwrap_or_default();
        let connection_state = if state_contents.contains("status='connected'") {
            ConnectionState::Connected
        } else {
            ConnectionState::Disconnected
        };

        return Ok((agent_state, connection_state));
    }
}

pub fn pause_agent() -> Result<()> {
    #[cfg(windows)]
    {
        Command::new(POWERSHELL_EXE)
            .args(["-Command", "Stop-Service", "-Name", "WazuhSvc"])
            .status()
            .context("failed to stop WazuhSvc")?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        Command::new(WAZUH_CONTROL_PATH)
            .arg("stop")
            .status()
            .context("failed to stop wazuh agent")?;
        return Ok(());
    }
}

pub fn restart_agent() -> Result<()> {
    #[cfg(windows)]
    {
        Command::new(POWERSHELL_EXE)
            .args(["-Command", "Stop-Service", "-Name", "WazuhSvc"])
            .status()
            .context("failed to stop WazuhSvc")?;
        Command::new(POWERSHELL_EXE)
            .args(["-Command", "Start-Service", "-Name", "WazuhSvc"])
            .status()
            .context("failed to start WazuhSvc")?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        Command::new(WAZUH_CONTROL_PATH)
            .arg("restart")
            .status()
            .context("failed to restart wazuh agent")?;
        return Ok(());
    }
}

pub fn update_agent() -> Result<()> {
    #[cfg(windows)]
    {
        Command::new(POWERSHELL_EXE)
            .args([
                "-Command",
                "Set-ExecutionPolicy",
                "-Scope",
                "CurrentUser",
                "-ExecutionPolicy",
                "RemoteSigned",
                "-Force",
            ])
            .status()
            .context("failed to set execution policy")?;
        Command::new(POWERSHELL_EXE)
            .args([
                "-Command",
                "&",
                "C:\\Program Files (x86)\\ossec-agent\\adorsys-update.ps1",
            ])
            .status()
            .context("failed to run update script")?;
        return Ok(());
    }

    #[cfg(not(windows))]
    {
        Command::new(UPDATE_SCRIPT_PATH)
            .status()
            .context("failed to run update script")?;
        return Ok(());
    }
}

pub fn get_local_version() -> Result<Option<String>> {
    let contents = std::fs::read_to_string(VERSION_FILE_PATH);
    match contents {
        Ok(value) => Ok(Some(value.trim().to_string())),
        Err(_) => Ok(None),
    }
}

pub async fn fetch_online_version(version_url: &str) -> Result<Option<String>> {
    let client = reqwest::Client::new();
    let response = client.get(version_url).send().await?;
    let body = response.text().await?;
    let trimmed = body.trim().to_string();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed))
    }
}
