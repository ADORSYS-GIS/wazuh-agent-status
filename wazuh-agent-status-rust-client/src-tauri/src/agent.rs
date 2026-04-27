use std::sync::Arc;
use tokio::sync::watch;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Inactive,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub total_memory: u64,
    pub used_memory: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub status: AgentStatus,
    pub connection: ConnectionStatus,
    pub version: String,
    pub tray_version: String,
    pub groups: Vec<String>,
    pub metrics: SystemMetrics,
    pub self_healing_enabled: bool,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status:       AgentStatus::Unknown,
            connection:   ConnectionStatus::Unknown,
            version:      "Unknown".to_string(),
            tray_version: "Unknown".to_string(),
            groups:       Vec::new(),
            metrics:      SystemMetrics {
                cpu_usage:    0.0,
                memory_usage: 0.0,
                total_memory: 0,
                used_memory:  0,
            },
            self_healing_enabled: true,
        }
    }
}

pub struct AgentManager {
    state_rx: watch::Receiver<AgentState>,
    server_addr: String,
}

impl AgentManager {
    pub fn new(default_addr: String) -> Arc<Self> {
        let (state_tx, state_rx) = watch::channel(AgentState::default());
        
        // Environment variable override takes precedence
        let server_addr = std::env::var("WAZUH_SERVER_ADDR")
            .unwrap_or_else(|_| default_addr);
            
        let addr_for_loop = server_addr.clone();
        
        // Spawn background task to keep the state in sync with the server
        tauri::async_runtime::spawn(async move {
            loop {
                if let Err(e) = run_sync_loop(addr_for_loop.clone(), state_tx.clone()).await {
                    eprintln!("Server sync loop error for {}: {}; retrying in 5s", addr_for_loop, e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        });

        Arc::new(Self { state_rx, server_addr })
    }

    pub fn get_state(&self) -> AgentState {
        self.state_rx.borrow().clone()
    }

    pub async fn check_updates(&self) -> anyhow::Result<serde_json::Value> {
        let mut stream = tokio::net::TcpStream::connect(&self.server_addr).await?;
        use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
        stream.write_all(b"get-version\n").await?;

        let mut reader = tokio::io::BufReader::new(stream);
        let mut line = String::new();
        if reader.read_line(&mut line).await? > 0 {
            if let Some(json) = line.strip_prefix("VERSION_CHECK: ") {
                let parsed: serde_json::Value = serde_json::from_str(json.trim())?;
                return Ok(parsed);
            }
        }
        Err(anyhow::anyhow!("Failed to get update info from server at {}", self.server_addr))
    }

    pub async fn run_update(&self, is_prerelease: bool) -> anyhow::Result<tokio::sync::mpsc::Receiver<String>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let server_addr = self.server_addr.clone();

        tokio::spawn(async move {
            let mut stream = match tokio::net::TcpStream::connect(&server_addr).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Failed to connect to server: {e}")).await;
                    return;
                }
            };

            use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
            let cmd = if is_prerelease { "update-prerelease\n" } else { "update\n" };
            if let Err(e) = stream.write_all(cmd.as_bytes()).await {
                let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Failed to send update command: {e}")).await;
                return;
            }

            // The 'update' command in the server triggers a background self-dial update stream.
            // However, it's actually easier for the client to just call 'initiate-update-stream' directly
            // if we want to pipe the logs back to the UI.
            // Let's call initiate-update-stream on a new connection.
            
            let stream = match tokio::net::TcpStream::connect(&server_addr).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(format!("UPDATE_PROGRESS: [FAILURE] Failed to connect for logs: {e}")).await;
                    return;
                }
            };
            let (reader, mut writer) = tokio::io::split(stream);
            let mut reader = tokio::io::BufReader::new(reader);
            
            let stream_cmd = if is_prerelease { "initiate-prerelease-update-stream\n" } else { "initiate-update-stream\n" };
            let _ = writer.write_all(stream_cmd.as_bytes()).await;

            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 { break; }
                let _ = tx.send(line.trim().to_string()).await;
                line.clear();
            }
        });

        Ok(rx)
    }

    pub async fn stream_logs(&self) -> anyhow::Result<tokio::sync::mpsc::Receiver<String>> {
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        let server_addr = self.server_addr.clone();

        tokio::spawn(async move {
            let mut stream = match tokio::net::TcpStream::connect(&server_addr).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(format!("[ERROR] Failed to connect to server for logs: {e}")).await;
                    return;
                }
            };

            use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
            if let Err(e) = stream.write_all(b"subscribe-logs\n").await {
                let _ = tx.send(format!("[ERROR] Failed to send log subscription: {e}")).await;
                return;
            }

            let mut reader = tokio::io::BufReader::new(stream);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 { break; }
                if let Some(json) = line.strip_prefix("LOG_LINE: ") {
                    let _ = tx.send(json.trim().to_string()).await;
                }
                line.clear();
            }
        });

        Ok(rx)
    }
}

async fn run_sync_loop(addr: String, tx: tokio::sync::watch::Sender<AgentState>) -> anyhow::Result<()> {
    // Connect to server
    let stream = tokio::net::TcpStream::connect(&addr).await?;
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = tokio::io::BufReader::new(reader);

    // Subscribe to status updates
    use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
    writer.write_all(b"subscribe-status\n").await?;

    let mut line = String::new();
    while reader.read_line(&mut line).await? > 0 {
        if let Some(json) = line.strip_prefix("STATUS_UPDATE: ") {
            if let Ok(state) = serde_json::from_str::<AgentState>(json.trim()) {
                let _ = tx.send(state);
            }
        }
        line.clear();
    }

    Ok(())
}
