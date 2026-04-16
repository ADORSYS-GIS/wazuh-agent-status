mod errors;
mod status_provider;
mod manager;
mod server;
mod version_utils;
mod group_extractor;
mod models;

use std::sync::Arc;
use tokio::time::Duration;
use tracing::{info, error};
use tracing_subscriber;

use crate::manager::AgentManager;
use crate::server::TcpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Wazuh Agent Status Rust Server...");

    let manager = Arc::new(AgentManager::new());
    
    // Start polling in the background
    let manager_clone = Arc::clone(&manager);
    tokio::spawn(async move {
        manager_clone.start_polling(Duration::from_secs(5)).await;
    });

    // Start TCP server
    let server = TcpServer::new("0.0.0.0:50505".to_string(), manager);
    
    // Handle graceful shutdown
    tokio::select! {
        res = server.start() => {
            if let Err(e) = res {
                error!("Server error: {:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal. Closing server...");
        }
    }

    info!("Server shutdown complete.");
    Ok(())
}
