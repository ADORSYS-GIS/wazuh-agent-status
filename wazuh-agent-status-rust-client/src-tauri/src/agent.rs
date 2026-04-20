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
        }
    }
}

pub struct AgentManager {
    state_rx: watch::Receiver<AgentState>,
}

impl AgentManager {
    pub fn new() -> Arc<Self> {
        let (state_tx, state_rx) = watch::channel(AgentState::default());
        
        // Spawn background task to keep the state in sync with the server
        tauri::async_runtime::spawn(async move {
            loop {
                if let Err(e) = run_sync_loop(state_tx.clone()).await {
                    eprintln!("Server sync loop error: {}; retrying in 5s", e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        });

        Arc::new(Self { state_rx })
    }

    pub fn get_state(&self) -> AgentState {
        self.state_rx.borrow().clone()
    }

    pub async fn check_updates(&self) -> anyhow::Result<serde_json::Value> {
        let mut stream = tokio::net::TcpStream::connect("127.0.0.1:50506").await?;
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
        Err(anyhow::anyhow!("Failed to get update info from server"))
    }
}

async fn run_sync_loop(tx: tokio::sync::watch::Sender<AgentState>) -> anyhow::Result<()> {
    // Connect to server (default addr)
    let stream = tokio::net::TcpStream::connect("127.0.0.1:50506").await?;
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
