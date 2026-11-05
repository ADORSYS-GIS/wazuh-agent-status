use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::watch;

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
    pub agent_id: String,
    pub agent_name: String,
    #[serde(skip_serializing)]
    pub agent_key: String,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status: AgentStatus::Unknown,
            connection: ConnectionStatus::Unknown,
            version: "Unknown".to_string(),
            tray_version: "Unknown".to_string(),
            groups: Vec::new(),
            metrics: SystemMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                total_memory: 0,
                used_memory: 0,
            },
            self_healing_enabled: true,
            agent_id: String::new(),
            agent_name: String::new(),
            agent_key: String::new(),
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
        let server_addr = std::env::var("WAZUH_SERVER_ADDR").unwrap_or(default_addr);

        let addr_for_loop = server_addr.clone();

        // Spawn background task to keep the state in sync with the server
        tauri::async_runtime::spawn(async move {
            loop {
                match run_sync_loop(addr_for_loop.clone(), state_tx.clone()).await {
                    Ok(()) => {
                        // Server closed the stream – reset to Unknown before retrying
                        let _ = state_tx.send(AgentState::default());
                        log::warn!("Sync loop ended for {}; reconnecting in 2s", addr_for_loop);
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                    Err(e) => {
                        // Connection failed – reset to Unknown before retrying
                        let _ = state_tx.send(AgentState::default());
                        log::error!(
                            "Server sync loop error for {}: {}; retrying in 5s",
                            addr_for_loop,
                            e
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Arc::new(Self {
            state_rx,
            server_addr,
        })
    }

    pub fn get_state(&self) -> AgentState {
        self.state_rx.borrow().clone()
    }

    pub fn subscribe(&self) -> watch::Receiver<AgentState> {
        self.state_rx.clone()
    }

    pub async fn check_updates(&self) -> anyhow::Result<serde_json::Value> {
        let mut stream = tokio::net::TcpStream::connect(&self.server_addr).await?;
        stream.write_all(b"get-version\n").await?;

        let mut reader = tokio::io::BufReader::new(stream);
        let mut line = String::new();
        if reader.read_line(&mut line).await? > 0
            && let Some(json) = line.strip_prefix("VERSION_CHECK: ")
        {
            let parsed: serde_json::Value = serde_json::from_str(json.trim())?;
            return Ok(parsed);
        }
        Err(anyhow::anyhow!(
            "Failed to get update info from server at {}",
            self.server_addr
        ))
    }

    pub async fn run_update(
        &self,
        is_prerelease: bool,
    ) -> anyhow::Result<tokio::sync::mpsc::Receiver<String>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let server_addr = self.server_addr.clone();

        tokio::spawn(async move {
            let mut stream = match tokio::net::TcpStream::connect(&server_addr).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx
                        .send(format!(
                            "UPDATE_PROGRESS: [FAILURE] Failed to connect to server: {e}"
                        ))
                        .await;
                    return;
                }
            };

            let cmd = if is_prerelease {
                "update-prerelease\n"
            } else {
                "update\n"
            };
            if let Err(e) = stream.write_all(cmd.as_bytes()).await {
                let _ = tx
                    .send(format!(
                        "UPDATE_PROGRESS: [FAILURE] Failed to send update command: {e}"
                    ))
                    .await;
                return;
            }

            // Read the streaming response from the server (same pattern as Go client)
            let mut reader = tokio::io::BufReader::new(stream);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
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
                    let _ = tx
                        .send(format!("[ERROR] Failed to connect to server for logs: {e}"))
                        .await;
                    return;
                }
            };

            if let Err(e) = stream.write_all(b"subscribe-logs\n").await {
                let _ = tx
                    .send(format!("[ERROR] Failed to send log subscription: {e}"))
                    .await;
                return;
            }

            let mut reader = tokio::io::BufReader::new(stream);
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                if let Some(json) = line.strip_prefix("LOG_LINE: ") {
                    let _ = tx.send(json.trim().to_string()).await;
                }
                line.clear();
            }
        });

        Ok(rx)
    }
}

async fn run_sync_loop(
    addr: String,
    tx: tokio::sync::watch::Sender<AgentState>,
) -> anyhow::Result<()> {
    log::info!("Connecting to Wazuh status server at {}...", addr);

    let stream = tokio::net::TcpStream::connect(&addr).await?;
    log::info!("Connected to Wazuh status server at {}", addr);

    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = tokio::io::BufReader::new(reader);

    writer.write_all(b"subscribe-status\n").await?;
    log::info!("Sent subscribe-status command");

    let mut line = String::new();
    while reader.read_line(&mut line).await? > 0 {
        if let Some(json) = line.strip_prefix("STATUS_UPDATE: ") {
            match serde_json::from_str::<AgentState>(json.trim()) {
                Ok(state) => {
                    let _ = tx.send(state.clone());
                    log::debug!("Received state update: {:?}", state.status);
                }
                Err(e) => {
                    log::warn!(
                        "Failed to parse STATUS_UPDATE JSON: {} — payload: {}",
                        e,
                        json.trim()
                    );
                }
            }
        } else {
            log::debug!("Ignoring non-status line: {}", line.trim());
        }
        line.clear();
    }

    log::warn!("Server closed status stream for {}", addr);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_agent_state_defaults() {
        let state = AgentState::default();
        assert!(matches!(state.status, AgentStatus::Unknown));
        assert!(matches!(state.connection, ConnectionStatus::Unknown));
        assert_eq!(state.version, "Unknown");
        assert_eq!(state.tray_version, "Unknown");
        assert!(state.self_healing_enabled);
        assert!(state.groups.is_empty());
    }

    #[tokio::test]
    async fn test_run_sync_loop_and_receive_state() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let (state_tx, state_rx) = tokio::sync::watch::channel(AgentState::default());

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut cmd = String::new();
            let _ = reader.read_line(&mut cmd).await;
            assert_eq!(cmd.trim(), "subscribe-status");

            let json = r#"{"status":"Active","connection":"Connected","version":"4.8.0","tray_version":"1.2.0","groups":[],"metrics":{"cpu_usage":0.0,"memory_usage":0.0,"total_memory":0,"used_memory":0},"self_healing_enabled":true,"agent_id":"001","agent_name":"test","agent_key":"test_key"}"#;
            let msg = format!("STATUS_UPDATE: {json}\n");
            let _ = writer.write_all(msg.as_bytes()).await;
            let _ = writer.flush().await;
        });

        // Run sync loop - will exit once the mock server finishes
        let _ = run_sync_loop(addr, state_tx).await;
        let current = state_rx.borrow().clone();
        assert!(matches!(current.status, AgentStatus::Active));
        assert!(matches!(current.connection, ConnectionStatus::Connected));
        assert_eq!(current.version, "4.8.0");
    }

    #[tokio::test]
    async fn test_check_updates_and_run_update() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let manager = AgentManager {
            state_rx: tokio::sync::watch::channel(AgentState::default()).1,
            server_addr: addr.clone(),
        };

        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let (reader, mut writer) = socket.split();
                    let mut reader = BufReader::new(reader);
                    let mut cmd = String::new();
                    if reader.read_line(&mut cmd).await.is_ok() {
                        let trimmed = cmd.trim();
                        if trimmed == "get-version" {
                            let resp = "VERSION_CHECK: {\"has_updates\": true}\n";
                            let _ = writer.write_all(resp.as_bytes()).await;
                        } else if trimmed == "update" || trimmed == "update-prerelease" {
                            let _ = writer
                                .write_all(b"UPDATE_PROGRESS: [STATUS] Downloading...\n")
                                .await;
                            let _ = writer
                                .write_all(b"UPDATE_PROGRESS: [SUCCESS] Updated\n")
                                .await;
                        } else if trimmed == "subscribe-logs" {
                            let _ = writer
                                .write_all(b"LOG_LINE: {\"raw\": \"test line\"}\n")
                                .await;
                        }
                    }
                });
            }
        });

        // 1. check_updates
        let updates = manager.check_updates().await.unwrap();
        assert_eq!(
            updates.get("has_updates").and_then(|v| v.as_bool()),
            Some(true)
        );

        // 2. run_update (stable)
        let mut rx = manager.run_update(false).await.unwrap();
        let l1 = rx.recv().await.unwrap();
        assert!(l1.contains("Downloading"));

        // 3. run_update (prerelease)
        let mut rx_pre = manager.run_update(true).await.unwrap();
        let l2 = rx_pre.recv().await.unwrap();
        assert!(l2.contains("Downloading"));

        // 4. stream_logs
        let mut log_rx = manager.stream_logs().await.unwrap();
        let log_msg = log_rx.recv().await.unwrap();
        assert!(log_msg.contains("test line"));

        // 5. get_state and subscribe
        let st = manager.get_state();
        assert_eq!(st.version, "Unknown");
        let _sub = manager.subscribe();
    }

    #[tokio::test]
    async fn test_agent_manager_new_and_disconnects() {
        let mgr = AgentManager::new("127.0.0.1:1".to_string());
        assert!(matches!(mgr.get_state().status, AgentStatus::Unknown));
        assert_eq!(mgr.get_state().version, "Unknown");
        let _sub = mgr.subscribe();

        // 1. check_updates connection failure
        let res = mgr.check_updates().await;
        assert!(res.is_err());

        // 2. run_update connection failure
        let mut rx = mgr.run_update(false).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert!(msg.contains("[FAILURE] Failed to connect to server"));

        // 3. stream_logs connection failure
        let mut log_rx = mgr.stream_logs().await.unwrap();
        let log_msg = log_rx.recv().await.unwrap();
        assert!(log_msg.contains("[ERROR] Failed to connect to server for logs"));
    }
}
