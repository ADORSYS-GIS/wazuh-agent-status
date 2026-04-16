use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;
use tracing::{info, error, warn};

use crate::manager::AgentManager;

pub struct TcpServer {
    addr: String,
    manager: Arc<AgentManager>,
}

impl TcpServer {
    pub fn new(addr: String, manager: Arc<AgentManager>) -> Self {
        Self { addr, manager }
    }

    pub async fn start(&self) -> tokio::io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        info!("Server listening on {}", self.addr);

        loop {
            let (socket, addr) = listener.accept().await?;
            info!("Accepted connection from {}", addr);
            
            let manager = Arc::clone(&self.manager);
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket, manager).await {
                    error!("Connection error: {:?}", e);
                }
            });
        }
    }
}

async fn handle_connection(mut socket: TcpStream, manager: Arc<AgentManager>) -> tokio::io::Result<()> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // Connection closed
        }

        let command = line.trim();
        info!("Received command: {}", command);

        match command {
            "get-version" => {
                let state = manager.get_state().await;
                let response = format!("VERSION_CHECK: {}\n", state.online_version_status);
                writer.write_all(response.as_bytes()).await?;
            }
            "subscribe-status" => {
                // Send initial state
                let state = manager.get_state().await;
                let initial_resp = format!("STATUS_UPDATE: {:?}, {:?}\n", state.status, state.connection);
                writer.write_all(initial_resp.as_bytes()).await?;

                // Start subscription loop
                let mut rx = manager.subscribe();
                loop {
                    tokio::select! {
                        result = rx.recv() => {
                            match result {
                                Ok(state) => {
                                    let update = format!("STATUS_UPDATE: {:?}, {:?}\n", state.status, state.connection);
                                    if let Err(e) = writer.write_all(update.as_bytes()).await {
                                        warn!("Failed to send update to client: {:?}", e);
                                        break;
                                    }
                                }
                                Err(broadcast::error::RecvError::Lagged(n)) => {
                                    warn!("Client lagged behind by {} messages", n);
                                }
                                Err(broadcast::error::RecvError::Closed) => {
                                    break;
                                }
                            }
                        }
                        // Check if client is still alive
                        check = reader.read_u8() => {
                            match check {
                                Ok(_) => {}, // Received data (heartbeat or command?)
                                Err(_) => break, // Disconnected
                            }
                        }
                    }
                }
                break;
            }
            "update" | "update-prerelease" | "initiate-update-stream" | "initiate-prerelease-update-stream" => {
                // Placeholder for update commands
                writer.write_all(b"ERROR: Update commands not yet implemented in Rust server\n").await?;
            }
            _ => {
                writer.write_all(format!("ERROR: Unknown command: {}\n", command).as_bytes()).await?;
            }
        }
    }

    info!("Connection closed");
    Ok(())
}
