//! TCP server — listens for client connections and dispatches commands.
//!
//! Supported commands (newline-terminated):
//!
//! | Command                              | Response                         |
//! |--------------------------------------|----------------------------------|
//! | `get-version`                        | `VERSION_CHECK: <status>`        |
//! | `subscribe-status`                   | streaming `STATUS_UPDATE: …`     |
//! | `update`                             | streaming `UPDATE_PROGRESS: …`   |
//! | `update-prerelease`                  | streaming `UPDATE_PROGRESS: …`   |
//! | `initiate-update-stream`             | streaming `UPDATE_PROGRESS: …`   |
//! | `initiate-prerelease-update-stream`  | streaming `UPDATE_PROGRESS: …`   |

use std::sync::Arc;

use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::time::timeout;
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
        let command = line.trim()
            .to_lowercase()
            .replace([' ', '_'], "-");

        if command.is_empty() { continue; }
        
        info!(command = %command, raw = %line.trim(), "Command received");

        match command.as_str() {
            // ── Version query ─────────────────────────────────────────────────
            "get-version" => {
                let status = manager.get_version_status().await;
                writer
                    .write_all(format!("VERSION_CHECK: {status}\n").as_bytes())
                    .await?;
                writer.flush().await?;
            }

            // ── Status subscription ───────────────────────────────────────────
            "subscribe-status" => {
                subscribe_status(&mut reader, &mut writer, &manager).await?;
                break; // Subscription ended — close connection
            }

            // ── Update commands (regular) ─────────────────────────────────────
            "update" | "initiate-update-stream" => {
                stream_update(&mut writer, &manager, false).await?;
                break; // Update stream finished — close connection
            }

            // ── Update commands (prerelease) ──────────────────────────────────
            "update-prerelease" | "initiate-prerelease-update-stream" => {
                stream_update(&mut writer, &manager, true).await?;
                break; // Update stream finished — close connection
            }

            // ── Unknown ───────────────────────────────────────────────────────
            unknown => {
                let msg = format!("ERROR: Unknown command: {unknown}\n");
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
    writer
        .write_all(
            format!("STATUS_UPDATE: {:?}, {:?}\n", state.status, state.connection).as_bytes(),
        )
        .await?;

    let mut rx = manager.subscribe();

    loop {
        tokio::select! {
            // New state broadcast received
            result = rx.recv() => {
                match result {
                    Ok(state) => {
                        let msg = format!(
                            "STATUS_UPDATE: {:?}, {:?}\n",
                            state.status, state.connection
                        );
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

/// Kick off an update and pipe `UPDATE_PROGRESS: …` lines back to the client
/// until the update finishes.
async fn stream_update<W>(
    writer: &mut W,
    manager: &AgentManager,
    prerelease: bool,
) -> tokio::io::Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Spawn the update so we can simultaneously stream its output.
    let manager_ref = manager;
    let update_fut = manager_ref.run_update(prerelease, tx);
    tokio::pin!(update_fut);

    loop {
        tokio::select! {
            // Drive the update to completion
            () = &mut update_fut => {
                // Drain any remaining messages
                while let Ok(msg) = rx.try_recv() {
                    writer.write_all(msg.as_bytes()).await?;
                }
                break;
            }
            // Forward progress messages to client as they arrive
            Some(msg) = rx.recv() => {
                writer.write_all(msg.as_bytes()).await?;
            }
        }
    }

    Ok(())
}
