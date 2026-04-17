use std::sync::Arc;
use std::time::Duration;
use wazuh_agent_status_rust_server::config::{AgentPaths, Config};
use wazuh_agent_status_rust_server::manager::AgentManager;
use wazuh_agent_status_rust_server::models::{AgentStatus, ConnectionStatus};
use wazuh_agent_status_rust_server::status_provider::StatusProvider;
use wazuh_agent_status_rust_server::errors::Result;

struct MockProvider {
    status: std::sync::Mutex<AgentStatus>,
}

impl StatusProvider for MockProvider {
    fn get_agent_status(&self) -> Result<AgentStatus> {
        Ok(self.status.lock().unwrap().clone())
    }
    fn get_connection_status(&self) -> Result<ConnectionStatus> {
        Ok(ConnectionStatus::Connected)
    }
    fn get_agent_version(&self) -> Result<String> {
        Ok("4.7.2".to_string())
    }
    fn get_agent_groups(&self) -> Result<Vec<String>> {
        Ok(vec!["test".to_string()])
    }
}

#[tokio::test]
async fn test_manager_polling_notification() {
    let config = Arc::new(Config {
        poll_interval: Duration::from_millis(100),
        ..Config::default()
    });
    let paths = Arc::new(AgentPaths::native());
    
    let mock_provider = Box::new(MockProvider {
        status: std::sync::Mutex::new(AgentStatus::Inactive),
    });
    
    let manager = Arc::new(AgentManager::new_custom(
        config,
        paths,
        mock_provider,
    ));
    
    let mut rx = manager.subscribe();
    
    // Start polling in background
    let manager_clone = Arc::clone(&manager);
    tokio::spawn(async move {
        manager_clone.start_polling().await;
    });
    
    // First tick should broadcast the state from provider.
    let state1 = rx.recv().await.unwrap();
    assert_eq!(state1.status, AgentStatus::Inactive);
}
