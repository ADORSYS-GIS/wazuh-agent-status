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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AgentPaths, Config};
    use crate::errors::ServerError;
    use crate::models::{AgentState, AgentStatus, ConnectionStatus, SystemMetrics};
    use crate::status_provider::StatusProvider;
    use tempfile::tempdir;

    struct FixedProvider(AgentState);

    impl StatusProvider for FixedProvider {
        fn get_agent_status(&self) -> Result<AgentStatus, ServerError> {
            Ok(self.0.status.clone())
        }
        fn get_connection_status(&self) -> Result<ConnectionStatus, ServerError> {
            Ok(self.0.connection.clone())
        }
        fn get_agent_version(&self) -> Result<String, ServerError> {
            Ok(self.0.version.clone())
        }
        fn get_tray_version(&self) -> Result<String, ServerError> {
            Ok(self.0.tray_version.clone())
        }
        fn get_agent_groups(&self) -> Result<Vec<String>, ServerError> {
            Ok(self.0.groups.clone())
        }
        fn get_system_metrics(&self) -> Result<SystemMetrics, ServerError> {
            Ok(self.0.metrics.clone())
        }
        fn get_partial_state(&self) -> Result<AgentState, ServerError> {
            Ok(self.0.clone())
        }
    }

    fn test_paths(dir: &std::path::Path) -> AgentPaths {
        AgentPaths {
            merged_mg: dir.join("merged.mg"),
            state_file: dir.join("wazuh-agentd.state"),
            version_json: dir.join("version.json"),
            version_file: dir.join("version.txt"),
            pid_file: dir.join("wazuh-agentd.pid"),
            update_script: dir.join("update.sh"),
            wazuh_control: dir.join("wazuh-control"),
            ossec_log: dir.join("ossec.log"),
            client_keys: dir.join("client.keys"),
            active_response_log: dir.join("active-responses.log"),
        }
    }

    fn test_manager(dir: &std::path::Path, max_connections: usize) -> Arc<AgentManager> {
        let config = Arc::new(Config {
            // Unreachable endpoints: version checks fail fast, never hit the network
            version_url: "http://127.0.0.1:9/versions.json".to_string(),
            stable_version_url: "http://127.0.0.1:9/version.txt".to_string(),
            poll_interval: Duration::from_millis(50),
            auto_update_check_interval: Duration::from_secs(3600),
            max_connections,
            self_healing: false,
            ..Config::default()
        });
        let provider = Box::new(FixedProvider(AgentState {
            status: AgentStatus::Active,
            connection: ConnectionStatus::Connected,
            version: "4.8.0".to_string(),
            tray_version: "1.0.0".to_string(),
            ..AgentState::default()
        }));
        Arc::new(AgentManager::new_custom(
            config,
            Arc::new(test_paths(dir)),
            provider,
        ))
    }

    async fn read_line_with_timeout<R: tokio::io::AsyncBufRead + Unpin>(
        lines: &mut tokio::io::Lines<R>,
    ) -> Option<String> {
        timeout(Duration::from_secs(5), lines.next_line())
            .await
            .ok()
            .and_then(|r| r.ok())
            .flatten()
    }

    #[tokio::test]
    async fn test_subscribe_status_receives_live_update() {
        let dir = tempdir().unwrap();
        let manager = test_manager(dir.path(), 4);

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server_manager = Arc::clone(&manager);
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            handle_connection(socket, server_manager).await
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        stream.write_all(b"subscribe_status\n").await.unwrap();
        let (r, mut w) = stream.into_split();
        let mut lines = BufReader::new(r).lines();

        // Initial snapshot (default state before polling)
        let first = read_line_with_timeout(&mut lines).await.unwrap();
        assert!(first.starts_with("STATUS_UPDATE: "), "{first}");
        assert!(!first.contains("4.8.0"));

        // Start polling so the manager broadcasts the provider's state
        tokio::time::sleep(Duration::from_millis(50)).await;
        let poll_manager = Arc::clone(&manager);
        let poller = tokio::spawn(async move { poll_manager.start_polling().await });

        let update = read_line_with_timeout(&mut lines).await.unwrap();
        assert!(update.starts_with("STATUS_UPDATE: "), "{update}");
        assert!(update.contains("4.8.0"), "{update}");

        // Bytes from the client are ignored; closing the socket ends the subscription
        w.write_all(b"x").await.unwrap();
        drop(w);
        drop(lines);
        let res = timeout(Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap();
        assert!(res.is_ok());
        poller.abort();
    }

    #[tokio::test]
    async fn test_tcp_server_drops_connections_over_limit() {
        let dir = tempdir().unwrap();
        let manager = test_manager(dir.path(), 1);

        // Reserve a free port for the server
        let port = {
            let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().port()
        };
        let addr = format!("127.0.0.1:{port}");
        let server = TcpServer::new(addr.clone(), manager);
        let handle = tokio::spawn(async move { server.start().await });

        // Wait for the listener to come up
        let mut first = None;
        for _ in 0..50 {
            if let Ok(s) = TcpStream::connect(&addr).await {
                first = Some(s);
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let mut first = first.expect("server should accept connections");

        // First client holds the only permit via a long-lived subscription
        first.write_all(b"subscribe-status\n").await.unwrap();
        let mut first_lines = BufReader::new(&mut first).lines();
        let snapshot = read_line_with_timeout(&mut first_lines).await.unwrap();
        assert!(snapshot.starts_with("STATUS_UPDATE: "));

        // Second client is accepted then immediately dropped (EOF)
        let mut second = TcpStream::connect(&addr).await.unwrap();
        let _ = second.write_all(b"get-version\n").await;
        let mut buf = Vec::new();
        let n = timeout(Duration::from_secs(5), second.read_to_end(&mut buf))
            .await
            .expect("second connection should be closed by the server")
            .unwrap_or(0);
        assert_eq!(n, 0, "over-limit connection must receive no data");

        handle.abort();
    }
}
