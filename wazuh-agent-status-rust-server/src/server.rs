//! TCP server — listens for client connections and dispatches commands.
//!
//! Supported commands (newline-terminated):
//!
//! | Command                              | Response                         |
//! |--------------------------------------|----------------------------------|
//! | `get-version`                        | `VERSION_CHECK: <status>`        |
//! | `subscribe-status`                   | streaming `STATUS_UPDATE: …`     |

use std::sync::Arc;

use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::time::{self, timeout};
use tracing::{info, warn, error};

use crate::manager::AgentManager;

// ── TcpServer ─────────────────────────────────────────────────────────────────

pub struct TcpServer {
    addr: String,
    manager: Arc<AgentManager>,
    /// Limits concurrent connections to prevent resource exhaustion.
    limit: Arc<tokio::sync::Semaphore>,
}

impl TcpServer {
    /// Create a new server bound to `addr` using the provided `manager`.
    pub fn new(addr: String, manager: Arc<AgentManager>) -> Self {
        let max_conns = manager.config().max_connections;
        Self { 
            addr, 
            manager,
            limit: Arc::new(tokio::sync::Semaphore::new(max_conns)),
        }
    }

    /// Accept connections in a loop.  Each connection is handled in its own
    /// Tokio task so one slow client never blocks others.
    pub async fn start(&self) -> tokio::io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        info!(addr = %self.addr, "Server listening");

        loop {
            let (socket, peer_addr) = listener.accept().await?;
            info!(peer = %peer_addr, "Accepted connection");

            // Acquire permit AFTER accept so we don't block the listener loop.
            // This also allows us to drop the connection gracefully if we're full.
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

                // Enable TCP keepalives to detect dead/ghost peers faster
                let _ = socket.set_nodelay(true);
                
                if let Err(e) = handle_connection(socket, manager).await {
                    error!(error = %e, peer = %peer_addr, "Connection handler error");
                }
            });
        }
    }
}

// ── Connection handler ────────────────────────────────────────────────────────

async fn handle_connection(
    mut socket: TcpStream,
    manager: Arc<AgentManager>,
) -> tokio::io::Result<()> {
    // ── Robustness: Max 2KB per command line to prevent DoS ───────────────────
    const MAX_LINE_LENGTH: usize = 2048;
    const IDLE_TIMEOUT: Duration = Duration::from_secs(60);

    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        
        // Wrap the read operation in a timeout to prevent leaked/hung connections.
        let read_result = timeout(IDLE_TIMEOUT, async {
            let mut handle = (&mut reader).take(MAX_LINE_LENGTH as u64);
            let bytes = handle.read_line(&mut line).await?;
            Ok::<usize, tokio::io::Error>(bytes)
        }).await;

        let bytes = match read_result {
            Ok(Ok(n)) => n,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                warn!("Connection timed out after {}s of inactivity", IDLE_TIMEOUT.as_secs());
                let _ = writer.write_all(b"ERROR: Connection idle timeout\n").await;
                let _ = writer.flush().await;
                break;
            }
        };

        if bytes == 0 {
            break; // Client closed connection
        }

        // Lenient command parsing: lowercase and treat space/underscore as hyphen.
        let raw_command = line.trim();
        let normalized = raw_command.to_lowercase()
            .replace([' ', '_'], "-");

        if normalized.is_empty() { continue; }
        
        info!(command = %normalized, raw = %raw_command, "Command received");

        match normalized.as_str() {
            // ── Version query ─────────────────────────────────────────────────
            "get-version" => {
                let status = manager.get_version_status().await;
                let json = serde_json::to_string(&status).unwrap_or_else(|_| "{}".to_string());
                info!(status = ?status, "Version check complete; sending response");
                writer
                    .write_all(format!("VERSION_CHECK: {json}\n").as_bytes())
                    .await?;
                writer.flush().await?;
            }

            // ── Status subscription ───────────────────────────────────────────
            "subscribe-status" => {
                subscribe_status(&mut reader, &mut writer, &manager).await?;
                break; // Subscription ended — close connection
            }

            // ── Log streaming ─────────────────────────────────────────────────
            "subscribe-logs" => {
                handle_log_stream(&mut writer, &manager).await?;
                break;
            }

            // ── Update triggers (compatible with Go protocol) ─────────────────
            "update" => {
                start_update_stream_async(&manager, false).await;
                writer.write_all(b"OK: Update process initiated\n").await?;
            }

            "update-prerelease" => {
                start_update_stream_async(&manager, true).await;
                writer.write_all(b"OK: Prerelease update process initiated\n").await?;
            }

            // ── Update streams ────────────────────────────────────────────────
            "initiate-update-stream" => {
                handle_update_stream(&mut writer, &manager, false).await?;
                break;
            }

            "initiate-prerelease-update-stream" => {
                handle_update_stream(&mut writer, &manager, true).await?;
                break;
            }

            // ── Unknown ───────────────────────────────────────────────────────
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

// ── Command implementations ───────────────────────────────────────────────────

/// Handle a `subscribe-status` session: push the current state immediately,
/// then keep pushing on every state change until the client disconnects.
async fn subscribe_status<R, W>(
    reader: &mut BufReader<R>,
    writer: &mut W,
    manager: &AgentManager,
) -> tokio::io::Result<()>
where
    R: tokio::io::AsyncRead + Unpin,
    W: AsyncWriteExt + Unpin,
{
    // Send current state immediately so the client isn't left waiting.
    let state = manager.get_state().await;
    let json = serde_json::to_string(&state).unwrap_or_default();
    writer
        .write_all(format!("STATUS_UPDATE: {json}\n").as_bytes())
        .await?;

    let mut rx = manager.subscribe();

    loop {
        tokio::select! {
            // New state broadcast received
            result = rx.recv() => {
                match result {
                    Ok(state) => {
                        let json = serde_json::to_string(&state).unwrap_or_default();
                        let msg = format!("STATUS_UPDATE: {json}\n");
                        if let Err(e) = writer.write_all(msg.as_bytes()).await {
                            warn!(error = %e, "Failed to write status update; dropping client");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(missed = n, "Client lagged; some updates were dropped");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }

            // Detect client disconnect — any byte received (or EOF) ends the sub
            check = reader.read_u8() => {
                match check {
                    Ok(_) => {} // Ignore any heartbeat bytes
                    Err(_) => break, // Client disconnected
                }
            }
        }
    }

    Ok(())
}

/// Handle a `initiate-update-stream` session: call the manager to start the
/// script execution and pipe all output back to the client.
async fn handle_update_stream<W>(
    writer: &mut W,
    manager: &AgentManager,
    is_prerelease: bool,
) -> tokio::io::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let mut rx = manager.initiate_update(is_prerelease).await;

    while let Some(line) = rx.recv().await {
        if let Err(e) = writer.write_all(format!("{line}\n").as_bytes()).await {
            warn!(error = %e, "Failed to write update log to client; stopping stream");
            break;
        }
        let _ = writer.flush().await;
    }

    Ok(())
}

/// Handle a `subscribe-logs` session: tail `ossec.log` in real-time and
/// pipe structured JSON lines back to the client.
async fn handle_log_stream<W>(
    writer: &mut W,
    manager: &AgentManager,
) -> tokio::io::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let mut rx = manager.stream_logs().await;

    while let Some(log_line) = rx.recv().await {
        let json = serde_json::to_string(&log_line).unwrap_or_default();
        let msg = format!("LOG_LINE: {json}\n");
        if let Err(e) = writer.write_all(msg.as_bytes()).await {
            warn!(error = %e, "Failed to write log line to client; stopping stream");
            break;
        }
        let _ = writer.flush().await;
    }

    Ok(())
}

/// Compatibility helper: dials the server's own listener to start a streaming
/// update session. This mirrors the Go implementation's behavior.
async fn start_update_stream_async(manager: &AgentManager, is_prerelease: bool) {
    let listen_addr = manager.config().listen_addr.clone();
    
    // Parse the port from the listen address
    let port = listen_addr.split(':').last().unwrap_or("50505");
    let addr = format!("127.0.0.1:{}", port);
    
    tokio::spawn(async move {
        // Give the current connection a moment to close/return if needed,
        // though not strictly required by the protocol.
        time::sleep(Duration::from_millis(100)).await;

        match TcpStream::connect(&addr).await {
            Ok(mut stream) => {
                let cmd = if is_prerelease {
                    "initiate-prerelease-update-stream\n"
                } else {
                    "initiate-update-stream\n"
                };
                if let Err(e) = stream.write_all(cmd.as_bytes()).await {
                    error!(error = %e, "Failed to send streaming command to self");
                    return;
                }
                
                // Keep the connection alive while the update happens.
                // We don't need to read anything here; the server-side task
                // of this new connection will handle the stream.
                let mut buf = [0u8; 1024];
                while let Ok(n) = stream.read(&mut buf).await {
                    if n == 0 { break; }
                }
            }
            Err(e) => {
                error!(error = %e, addr = %addr, "Failed to dial self for update stream");
            }
        }
    });
}
