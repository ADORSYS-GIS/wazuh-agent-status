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
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;
use tracing::{info, warn, error};

use crate::manager::AgentManager;

// ── TcpServer ─────────────────────────────────────────────────────────────────

pub struct TcpServer {
    addr: String,
    manager: Arc<AgentManager>,
    /// Limits concurrent connections to prevent resource exhaustion.
    limit: Arc<tokio::sync::Semaphore>,
    /// TLS acceptor for mTLS.
    acceptor: TlsAcceptor,
}

impl TcpServer {
    /// Create a new server bound to `addr` using the provided `manager` and `acceptor`.
    pub fn new(addr: String, manager: Arc<AgentManager>, acceptor: TlsAcceptor) -> Self {
        let max_conns = manager.config().max_connections;
        Self { 
            addr, 
            manager,
            limit: Arc::new(tokio::sync::Semaphore::new(max_conns)),
            acceptor,
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
            let acceptor = self.acceptor.clone();
            tokio::spawn(async move {
                let _permit = permit;

                // ── TLS Handshake ───────────────────────────────────────────
                let socket = match acceptor.accept(socket).await {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(error = %e, peer = %peer_addr, "TLS handshake failed");
                        return;
                    }
                };

                // Enable TCP keepalives to detect dead/ghost peers faster
                // Note: tokio-rustls TlsStream doesn't have set_nodelay; 
                // it's already set on the underlying socket before accept in start loop if needed,
                // but we'll stick to basic stream handling for now.
                
                if let Err(e) = handle_connection(socket, manager).await {
                    error!(error = %e, peer = %peer_addr, "Connection handler error");
                }
            });
        }
    }
}

// ── Connection handler ────────────────────────────────────────────────────────

async fn handle_connection<S>(
    socket: S,
    manager: Arc<AgentManager>,
) -> tokio::io::Result<()> 
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    // ── Robustness: Max 2KB per command line to prevent DoS ───────────────────
    const MAX_LINE_LENGTH: usize = 2048;
    const IDLE_TIMEOUT: Duration = Duration::from_secs(60);

    let (reader, mut writer) = tokio::io::split(socket);
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
                let _ = tokio::io::AsyncWriteExt::write_all(&mut writer, b"ERROR: Connection idle timeout\n").await;
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
                let msg = format!("VERSION_CHECK: {json}\n");
                tokio::io::AsyncWriteExt::write_all(&mut writer, msg.as_bytes()).await?;
                tokio::io::AsyncWriteExt::flush(&mut writer).await?;
            }

            // ── Status subscription ───────────────────────────────────────────
            "subscribe-status" => {
                subscribe_status(&mut reader, &mut writer, &manager).await?;
                break; // Subscription ended — close connection
            }

            // ── Unknown ───────────────────────────────────────────────────────
            _ => {
                let msg = format!("ERROR: Unknown command: {raw_command}\n");
                let _ = tokio::io::AsyncWriteExt::write_all(&mut writer, msg.as_bytes()).await;
                let _ = tokio::io::AsyncWriteExt::flush(&mut writer).await;
            }
        }
    }

    let _ = writer.shutdown().await;
    info!("Connection closed gracefully");
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
    let msg = format!("STATUS_UPDATE: {json}\n");
    tokio::io::AsyncWriteExt::write_all(writer, msg.as_bytes()).await?;
    tokio::io::AsyncWriteExt::flush(writer).await?;

    let mut rx = manager.subscribe();

    loop {
        tokio::select! {
            // New state broadcast received
            result = rx.recv() => {
                match result {
                    Ok(state) => {
                        let json = serde_json::to_string(&state).unwrap_or_default();
                        let msg = format!("STATUS_UPDATE: {json}\n");
                        if let Err(e) = tokio::io::AsyncWriteExt::write_all(writer, msg.as_bytes()).await {
                            warn!(error = %e, "Failed to write status update; dropping client");
                            break;
                        }
                        let _ = tokio::io::AsyncWriteExt::flush(writer).await;
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
