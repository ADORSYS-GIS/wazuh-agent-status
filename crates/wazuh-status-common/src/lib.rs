use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use wazuh_status_proto_build::wazuh_status::wazuh_status_client::WazuhStatusClient;
use wazuh_status_proto_build::wazuh_status::{
    ActionReply, Empty, StatusReply, UpdateStatusReply, VersionReply,
};
use wazuh_status_socket::SocketConfig;

#[derive(Clone)]
pub struct WazuhClient {
    config: SocketConfig,
    channel: Arc<Mutex<Option<Channel>>>,
}

impl WazuhClient {
    pub fn new(config: SocketConfig) -> Self {
        Self {
            config,
            channel: Arc::new(Mutex::new(None)),
        }
    }

    pub fn default() -> Self {
        Self::new(default_socket_config())
    }

    pub async fn get_status(&self) -> Result<StatusReply> {
        let mut client = self.grpc_client().await?;
        Ok(client.get_status(Empty {}).await?.into_inner())
    }

    pub async fn pause(&self) -> Result<ActionReply> {
        let mut client = self.grpc_client().await?;
        Ok(client.pause(Empty {}).await?.into_inner())
    }

    pub async fn restart(&self) -> Result<ActionReply> {
        let mut client = self.grpc_client().await?;
        Ok(client.restart(Empty {}).await?.into_inner())
    }

    pub async fn start_update(&self) -> Result<ActionReply> {
        let mut client = self.grpc_client().await?;
        Ok(client.start_update(Empty {}).await?.into_inner())
    }

    pub async fn get_update_status(&self) -> Result<UpdateStatusReply> {
        let mut client = self.grpc_client().await?;
        Ok(client.get_update_status(Empty {}).await?.into_inner())
    }

    pub async fn check_version(&self) -> Result<VersionReply> {
        let mut client = self.grpc_client().await?;
        Ok(client.check_version(Empty {}).await?.into_inner())
    }

    async fn grpc_client(&self) -> Result<WazuhStatusClient<Channel>> {
        let channel = self.channel().await?;
        Ok(WazuhStatusClient::new(channel))
    }

    async fn channel(&self) -> Result<Channel> {
        let mut guard = self.channel.lock().await;
        if let Some(channel) = guard.as_ref() {
            return Ok(channel.clone());
        }
        let channel = wazuh_status_socket::connect_channel(&self.config).await?;
        *guard = Some(channel.clone());
        Ok(channel)
    }
}

pub fn default_socket_config() -> SocketConfig {
    #[cfg(windows)]
    {
        SocketConfig::TcpLoopback { port: 50505 }
    }
    #[cfg(not(windows))]
    {
        SocketConfig::Unix {
            path: "/var/run/wazuh-status/wazuh-status.sock".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::default_socket_config;
    use wazuh_status_socket::SocketConfig;

    #[test]
    fn default_socket_config_is_loopback_on_windows_or_uds_on_unix() {
        let config = default_socket_config();
        #[cfg(windows)]
        assert!(matches!(config, SocketConfig::TcpLoopback { .. }));
        #[cfg(not(windows))]
        assert!(matches!(config, SocketConfig::Unix { .. }));
    }
}
