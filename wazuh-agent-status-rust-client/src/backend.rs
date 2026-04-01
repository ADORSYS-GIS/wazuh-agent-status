use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tracing::{error, info, warn};

pub struct BackendClient {
    address: String,
}

impl BackendClient {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
        }
    }

    pub async fn monitor_status<F>(&self, on_update: F) -> Result<()>
    where
        F: Fn(String, String),
    {
        loop {
            match self.connect_and_subscribe().await {
                Ok(mut reader) => {
                    info!("Connected to backend.");
                    let mut line = String::new();
                    loop {
                        match reader.read_line(&mut line).await {
                            Ok(0) => {
                                warn!("Backend closed connection.");
                                break;
                            }
                            Ok(_) => {
                                let trimmed = line.trim();
                                if let Some(data) = trimmed.strip_prefix("STATUS_UPDATE: ") {
                                    let parts: Vec<&str> = data.splitn(2, ", ").collect();
                                    if parts.len() == 2 {
                                        on_update(parts[0].to_string(), parts[1].to_string());
                                    }
                                }
                                line.clear();
                            }
                            Err(e) => {
                                error!("Error reading from backend: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to connect to backend: {}. Retrying...", e);
                    on_update("Unknown".to_string(), "Disconnected".to_string());
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    async fn connect_and_subscribe(&self) -> Result<BufReader<TcpStream>> {
        let mut stream = TcpStream::connect(&self.address)
            .await
            .context("Failed to connect to backend")?;

        stream.write_all(b"subscribe-status\n").await?;
        Ok(BufReader::new(stream))
    }
}
