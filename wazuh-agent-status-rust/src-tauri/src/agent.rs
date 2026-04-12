use std::process::Command;
use std::path::PathBuf;
use crate::config::AppConfig;

#[derive(Debug, serde::Serialize)]
pub struct AgentStatus {
    pub status: String,
    pub connection: String,
    pub agent_version: String,
}

pub struct AgentManager {
    config: AppConfig,
}

impl AgentManager {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    fn get_wazuh_base_path(&self) -> PathBuf {
        #[cfg(target_os = "linux")]
        return PathBuf::from(&self.config.wazuh.linux_base_path);
        
        #[cfg(target_os = "macos")]
        return PathBuf::from(&self.config.wazuh.macos_base_path);
        
        #[cfg(target_os = "windows")]
        return PathBuf::from(&self.config.wazuh.windows_base_path);
    }

    fn get_agent_version(&self) -> String {
        let base_path = self.get_wazuh_base_path();
        let version_path = base_path.join("etc/version.txt");
        
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let output = Command::new("sudo")
                .arg("-n")
                .arg("cat")
                .arg(&version_path)
                .output();
            
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !stdout.is_empty() {
                    return stdout;
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("powershell.exe")
                .arg("-Command")
                .arg(format!("Get-Content -Path '{}'", version_path.display()))
                .output();
            
            if let Ok(output) = output {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }

        "Unknown".to_string()
    }

    pub fn check_status(&self) -> AgentStatus {
        let mut status = "Inactive".to_string();
        let mut connection = "Disconnected".to_string();
        let agent_version = self.get_agent_version();

        let base_path = self.get_wazuh_base_path();

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let control_path = base_path.join("bin/wazuh-control");
            let output = Command::new("sudo")
                .arg("-n")
                .arg(control_path)
                .arg("status")
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("wazuh-agentd is running") {
                    status = "Active".to_string();
                }
            }

            let state_path = base_path.join("var/run/wazuh-agentd.state");
            let conn_output = Command::new("sudo")
                .arg("-n")
                .arg("grep")
                .arg("^status")
                .arg(state_path)
                .output();

            if let Ok(output) = conn_output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("status='connected'") {
                    connection = "Connected".to_string();
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("powershell.exe")
                .arg("-Command")
                .arg("Get-Service -Name WazuhSvc")
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("Running") {
                    status = "Active".to_string();
                }
            }

            let state_path = base_path.join("wazuh-agent.state");
            let conn_output = Command::new("powershell.exe")
                .arg("-Command")
                .arg(format!("Select-String -Path '{}' -Pattern '^status'", state_path.display()))
                .output();

            if let Ok(output) = conn_output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("status='connected'") {
                    connection = "Connected".to_string();
                }
            }
        }

        AgentStatus { status, connection, agent_version }
    }
}
