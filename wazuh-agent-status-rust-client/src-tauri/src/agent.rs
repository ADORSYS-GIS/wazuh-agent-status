use std::sync::Arc;
use tokio::sync::watch;
use serde::{Deserialize, Serialize};
use crate::config::AppConfig;

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
    server_addr: String,
    secret_store: Arc<dyn crate::secret_store::SecretStore>,
}

impl AgentManager {
    pub fn new(config: Arc<AppConfig>, secret_store: Arc<dyn crate::secret_store::SecretStore>) -> Arc<Self> {
        let (state_tx, state_rx) = watch::channel(AgentState::default());
        
        let server_addr = std::env::var("WAZUH_SERVER_ADDR")
            .unwrap_or_else(|_| config.server_addr.clone());
            
        let addr_for_loop = server_addr.clone();
        let secret_store_inner = Arc::clone(&secret_store);
        
        // Spawn background task to keep the state in sync with the server
        tauri::async_runtime::spawn(async move {
            loop {
                if let Err(e) = run_sync_loop(addr_for_loop.clone(), state_tx.clone(), Arc::clone(&secret_store_inner)).await {
                    eprintln!("Server sync loop error for {}: {}; retrying in 5s", addr_for_loop, e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        });

        Arc::new(Self { state_rx, server_addr, secret_store })
    }

    pub fn get_state(&self) -> AgentState {
        self.state_rx.borrow().clone()
    }

    pub async fn check_updates(&self) -> anyhow::Result<serde_json::Value> {
        let stream = tokio::net::TcpStream::connect(&self.server_addr).await?;
        let connector = build_tls_connector(self.secret_store.as_ref())?;
        let stream = connector.connect(
            tokio_rustls::rustls::pki_types::ServerName::try_from("localhost")?.to_owned(),
            stream
        ).await?;

        use tokio::io::{AsyncWriteExt, AsyncBufReadExt};
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = tokio::io::BufReader::new(reader);
        writer.write_all(b"get-version\n").await?;
        writer.flush().await?;

        let mut line = String::new();
        let result = if reader.read_line(&mut line).await? > 0 {
            if let Some(json) = line.strip_prefix("VERSION_CHECK: ") {
                let parsed: serde_json::Value = serde_json::from_str(json.trim())?;
                Ok(parsed)
            } else {
                Err(anyhow::anyhow!("Failed to get update info from server at {}", self.server_addr))
            }
        } else {
            Err(anyhow::anyhow!("Connection closed prematurely by server at {}", self.server_addr))
        };

        let _ = writer.shutdown().await;
        result
    }
}

async fn run_sync_loop(addr: String, tx: tokio::sync::watch::Sender<AgentState>, secret_store: Arc<dyn crate::secret_store::SecretStore>) -> anyhow::Result<()> {
    // Connect to server
    let stream = tokio::net::TcpStream::connect(&addr).await?;
    let connector = build_tls_connector(secret_store.as_ref())?;
    let stream = connector.connect(
        tokio_rustls::rustls::pki_types::ServerName::try_from("localhost")?.to_owned(),
        stream
    ).await?;

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

    let _ = writer.shutdown().await;
    Ok(())
}

fn build_tls_connector(store: &dyn crate::secret_store::SecretStore) -> anyhow::Result<tokio_rustls::TlsConnector> {
    use std::sync::Arc;
    use tokio_rustls::rustls;

    let ca_certs = store.load_ca_certs()?;
    let client_certs = store.load_entity_certs()?;
    let client_key = store.load_private_key()?;

    let mut root_cert_store = rustls::RootCertStore::empty();
    for cert in ca_certs {
        root_cert_store.add(cert)?;
    }

    let tls_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(client_certs, client_key)?;

    Ok(tokio_rustls::TlsConnector::from(Arc::new(tls_config)))
}
