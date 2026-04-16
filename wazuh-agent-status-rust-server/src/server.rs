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

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tracing::{info, warn, error};

use crate::manager::AgentManager;

// ── TcpServer ─────────────────────────────────────────────────────────────────

pub struct TcpServer {
    addr: String,
    manager: Arc<AgentManager>,
}

impl TcpServer {
    pub fn new(addr: String, manager: Arc<AgentManager>) -> Self {
        Self { addr, manager }
    }

    /// Accept connections in a loop.  Each connection is handled in its own
    /// Tokio task so one slow client never blocks others.
    pub async fn start(&self) -> tokio::io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        info!(addr = %self.addr, "Server listening");

        loop {
            let (socket, peer_addr) = listener.accept().await?;
            info!(peer = %peer_addr, "Accepted connection");

            let manager = Arc::clone(&self.manager);
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket, manager).await {
                    error!(error = %e, "Connection handler error");
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

    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        
        // Read until newline with a cap on total bytes read
        let mut handle = reader.take(MAX_LINE_LENGTH as u64);
        if handle.read_line(&mut line).await? == 0 {
            break; // Client closed connection
        }
        reader = handle.into_inner();

        let command = line.trim().to_string();
        if command.is_empty() { continue; }
        
        info!(command = %command, "Command received");

        match command.as_str() {
            // ── Version query ─────────────────────────────────────────────────
            "get-version" => {
                let status = manager.get_version_status().await;
                writer
                    .write_all(format!("VERSION_CHECK: {status}\n").as_bytes())
                    .await?;
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
