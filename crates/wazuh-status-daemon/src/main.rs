use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use wazuh_core::{
    check_service_status, fetch_online_version, get_local_version, pause_agent, restart_agent,
    update_agent, AgentState, ConnectionState,
};
use wazuh_status_proto_build::wazuh_status::{
    wazuh_status_server::{WazuhStatus, WazuhStatusServer},
    ActionReply, AgentState as ProtoAgentState, ConnectionState as ProtoConnectionState, Empty,
    StatusReply, UpdateState, UpdateStatusReply, VersionReply, VersionState,
};
use wazuh_socket::{bind_incoming, SocketConfig};

const VERSION_URL: &str = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/main/version.txt";

#[derive(Parser, Debug)]
#[command(name = "wazuh-status-daemon")]
struct Args {
    #[cfg(not(windows))]
    #[arg(long, default_value = "/var/run/wazuh-status/wazuh-status.sock")]
    socket_path: String,

    #[cfg(windows)]
    #[arg(long, default_value_t = 50505)]
    port: u16,
}

#[derive(Debug, Clone)]
struct DaemonState {
    update_state: Arc<Mutex<UpdateState>>,
    update_message: Arc<Mutex<String>>,
}

#[derive(Debug, Clone)]
struct WazuhStatusService {
    state: DaemonState,
}

#[tonic::async_trait]
impl WazuhStatus for WazuhStatusService {
    async fn get_status(&self, _request: Request<Empty>) -> Result<Response<StatusReply>, Status> {
        let (agent_state, connection_state) =
            check_service_status().map_err(|err| Status::internal(err.to_string()))?;
        Ok(Response::new(StatusReply {
            agent_state: map_agent_state(agent_state) as i32,
            connection_state: map_connection_state(connection_state) as i32,
        }))
    }

    async fn pause(&self, _request: Request<Empty>) -> Result<Response<ActionReply>, Status> {
        pause_agent().map_err(|err| Status::internal(err.to_string()))?;
        Ok(Response::new(ActionReply {
            ok: true,
            message: "Paused the Wazuh Agent".to_string(),
        }))
    }

    async fn restart(&self, _request: Request<Empty>) -> Result<Response<ActionReply>, Status> {
        restart_agent().map_err(|err| Status::internal(err.to_string()))?;
        Ok(Response::new(ActionReply {
            ok: true,
            message: "Restarted the Wazuh Agent".to_string(),
        }))
    }

    async fn start_update(&self, _request: Request<Empty>) -> Result<Response<ActionReply>, Status> {
        {
            let mut state = self.state.update_state.lock().await;
            *state = UpdateState::UpdateStateInProgress;
        }
        {
            let mut message = self.state.update_message.lock().await;
            *message = "Updating...".to_string();
        }

        let state = self.state.clone();
        tokio::spawn(async move {
            let result = update_agent();
            let mut update_state = state.update_state.lock().await;
            let mut update_message = state.update_message.lock().await;
            match result {
                Ok(_) => {
                    *update_state = UpdateState::UpdateStateIdle;
                    *update_message = "Update finished".to_string();
                }
                Err(err) => {
                    *update_state = UpdateState::UpdateStateFailed;
                    *update_message = format!("Update failed: {err}");
                }
            }
        });

        Ok(Response::new(ActionReply {
            ok: true,
            message: "Update started".to_string(),
        }))
    }

    async fn get_update_status(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<UpdateStatusReply>, Status> {
        let state = self.state.update_state.lock().await;
        let message = self.state.update_message.lock().await;
        Ok(Response::new(UpdateStatusReply {
            update_state: *state as i32,
            message: message.clone(),
        }))
    }

    async fn check_version(&self, _request: Request<Empty>) -> Result<Response<VersionReply>, Status> {
        let local = get_local_version().map_err(|err| Status::internal(err.to_string()))?;
        let online = fetch_online_version(VERSION_URL)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let (state, version) = match (local, online) {
            (Some(local_version), Some(online_version)) => {
                if local_version == online_version {
                    (VersionState::VersionStateUpToDate, local_version)
                } else {
                    (VersionState::VersionStateOutdated, local_version)
                }
            }
            (Some(local_version), None) => (VersionState::VersionStateUnknown, local_version),
            (None, _) => (VersionState::VersionStateUnknown, "Unknown".to_string()),
        };

        Ok(Response::new(VersionReply {
            version_state: state as i32,
            version,
        }))
    }
}

fn map_agent_state(state: AgentState) -> ProtoAgentState {
    match state {
        AgentState::Active => ProtoAgentState::AgentStateActive,
        AgentState::Inactive => ProtoAgentState::AgentStateInactive,
        AgentState::Unknown => ProtoAgentState::AgentStateUnknown,
    }
}

fn map_connection_state(state: ConnectionState) -> ProtoConnectionState {
    match state {
        ConnectionState::Connected => ProtoConnectionState::ConnectionStateConnected,
        ConnectionState::Disconnected => ProtoConnectionState::ConnectionStateDisconnected,
        ConnectionState::Unknown => ProtoConnectionState::ConnectionStateUnknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{map_agent_state, map_connection_state};
    use wazuh_core::{AgentState, ConnectionState};
    use wazuh_proto_build::wazuh_status::{AgentState as ProtoAgentState, ConnectionState as ProtoConnectionState};

    #[test]
    fn map_agent_state_maps_correctly() {
        assert_eq!(map_agent_state(AgentState::Active), ProtoAgentState::AgentStateActive);
        assert_eq!(map_agent_state(AgentState::Inactive), ProtoAgentState::AgentStateInactive);
        assert_eq!(map_agent_state(AgentState::Unknown), ProtoAgentState::AgentStateUnknown);
    }

    #[test]
    fn map_connection_state_maps_correctly() {
        assert_eq!(
            map_connection_state(ConnectionState::Connected),
            ProtoConnectionState::ConnectionStateConnected
        );
        assert_eq!(
            map_connection_state(ConnectionState::Disconnected),
            ProtoConnectionState::ConnectionStateDisconnected
        );
        assert_eq!(
            map_connection_state(ConnectionState::Unknown),
            ProtoConnectionState::ConnectionStateUnknown
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let _log_guard = init_tracing()?;

    #[cfg(not(windows))]
    let socket_config = SocketConfig::Unix {
        path: args.socket_path.into(),
    };

    #[cfg(windows)]
    let socket_config = SocketConfig::TcpLoopback { port: args.port };

    let incoming = bind_incoming(&socket_config).await?;
    info!("daemon started");

    let service = WazuhStatusService {
        state: DaemonState {
            update_state: Arc::new(Mutex::new(UpdateState::UpdateStateIdle)),
            update_message: Arc::new(Mutex::new("Idle".to_string())),
        },
    };

    let shutdown = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!("shutdown signal error: {err}");
        }
        info!("shutdown signal received");
    };

    tonic::transport::Server::builder()
        .add_service(WazuhStatusServer::new(service))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await?;

    Ok(())
}

fn init_tracing() -> Result<tracing_appender::non_blocking::WorkerGuard, anyhow::Error> {
    let log_dir = log_dir();
    std::fs::create_dir_all(&log_dir)?;
    let file_appender = tracing_appender::rolling::daily(log_dir, "wazuh-agent-status.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let stdout_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);
    let file_layer = tracing_subscriber::fmt::layer().with_writer(non_blocking);
    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();
    Ok(guard)
}

fn log_dir() -> PathBuf {
    #[cfg(windows)]
    {
        let base = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string());
        PathBuf::from(base).join("wazuh").join("logs")
    }
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Logs")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        PathBuf::from("/var/log")
    }
}
