use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::time::timeout;
use tracing::{error, info, warn};

use crate::manager::AgentManager;

pub struct TcpServer {
    addr: String,
    manager: Arc<AgentManager>,
    limit: Arc<tokio::sync::Semaphore>,
}

impl TcpServer {
    pub fn new(addr: String, manager: Arc<AgentManager>) -> Self {
        let max_conns = manager.config().max_connections;
        Self {
            addr,
            manager,
            limit: Arc::new(tokio::sync::Semaphore::new(max_conns)),
        }
    }

    pub async fn start(&self) -> tokio::io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        info!(addr = %self.addr, "Server listening");

        loop {
            let (socket, peer_addr) = listener.accept().await?;
            info!(peer = %peer_addr, "Accepted connection");

            let permit = match Arc::clone(&self.limit).try_acquire_owned() {
                Ok(p) => p,
                Err(_) => {
                    let limit = self.manager.config().max_connections;
                    warn!(peer = %peer_addr, "Server full (limit {}); dropping connection", limit);
                    continue;
                }
            };

            let manager = Arc::clone(&self.manager);
            tokio::spawn(async move {
                let _permit = permit;
                let _ = socket.set_nodelay(true);

                if let Err(e) = handle_connection(socket, manager).await {
                    error!(error = %e, peer = %peer_addr, "Connection handler error");
                }
            });
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    manager: Arc<AgentManager>,
) -> tokio::io::Result<()> {
    const MAX_LINE_LENGTH: usize = 2048;
    const IDLE_TIMEOUT: Duration = Duration::from_secs(60);

    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();

        let read_result = timeout(IDLE_TIMEOUT, async {
            let mut handle = (&mut reader).take(MAX_LINE_LENGTH as u64);
            let bytes = handle.read_line(&mut line).await?;
            Ok::<usize, tokio::io::Error>(bytes)
        })
        .await;

        let bytes = match read_result {
            Ok(Ok(n)) => n,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                warn!("Connection timed out after {}s", IDLE_TIMEOUT.as_secs());
                let _ = writer.write_all(b"ERROR: Connection idle timeout\n").await;
                let _ = writer.flush().await;
                break;
            }
        };

        if bytes == 0 {
            break;
        }

        let raw_command = line.trim();
        let normalized = raw_command.to_lowercase().replace([' ', '_'], "-");

        if normalized.is_empty() {
            continue;
        }

        info!(command = %normalized, "Command received");

        match normalized.as_str() {
            "get-version" => {
                let status = manager.get_version_status().await;
                let json = serde_json::to_string(&status).unwrap_or_else(|_| "{}".to_string());
                writer
                    .write_all(format!("VERSION_CHECK: {json}\n").as_bytes())
                    .await?;
                writer.flush().await?;
            }

            "subscribe-status" => {
                subscribe_status(&mut reader, &mut writer, &manager).await?;
                break;
            }

            "subscribe-logs" => {
                handle_log_stream(&mut writer, &manager).await?;
                break;
            }

            "update" | "initiate-update-stream" => {
                handle_update_stream(&mut writer, &manager, false).await?;
                break;
            }

            "update-prerelease" | "initiate-prerelease-update-stream" => {
                handle_update_stream(&mut writer, &manager, true).await?;
                break;
            }

            _ => {
                let msg = format!("ERROR: Unknown command: {raw_command}\n");
                writer.write_all(msg.as_bytes()).await?;
                writer.flush().await?;
            }
        }
    }

    info!("Connection closed");
    Ok(())
}

async fn subscribe_status<R, W>(
    reader: &mut BufReader<R>,
    writer: &mut W,
    manager: &AgentManager,
) -> tokio::io::Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let state = manager.get_state().await;
    let json = serde_json::to_string(&state).unwrap_or_default();
    writer
        .write_all(format!("STATUS_UPDATE: {json}\n").as_bytes())
        .await?;

    let mut rx = manager.subscribe();

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(state) => {
                        let json = serde_json::to_string(&state).unwrap_or_default();
                        let msg = format!("STATUS_UPDATE: {json}\n");
                        if let Err(e) = writer.write_all(msg.as_bytes()).await {
                            warn!(error = %e, "Failed to write status update");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(missed = n, "Client lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }

            check = reader.read_u8() => {
                match check {
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }
    }

    Ok(())
}

async fn handle_update_stream<W>(
    writer: &mut W,
    manager: &AgentManager,
    is_prerelease: bool,
) -> tokio::io::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    info!(is_prerelease, "Starting update stream");
    let mut rx = manager.initiate_update(is_prerelease, true).await;

    loop {
        match rx.recv().await {
            Some(line) => {
                if let Err(e) = writer.write_all(format!("{line}\n").as_bytes()).await {
                    warn!(error = %e, "Failed to write update log");
                    break;
                }
                let _ = writer.flush().await;
            }
            None => {
                info!("Update channel closed");
                break;
            }
        }
    }

    info!("Update stream finished");
    Ok(())
}

async fn handle_log_stream<W>(writer: &mut W, manager: &AgentManager) -> tokio::io::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let mut rx = manager.stream_logs().await;

    while let Some(log_line) = rx.recv().await {
        let json = serde_json::to_string(&log_line).unwrap_or_default();
        let msg = format!("LOG_LINE: {json}\n");
        if let Err(e) = writer.write_all(msg.as_bytes()).await {
            warn!(error = %e, "Failed to write log line");
            break;
        }
        let _ = writer.flush().await;
    }

    Ok(())
}
