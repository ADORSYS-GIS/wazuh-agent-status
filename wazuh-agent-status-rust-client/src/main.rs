mod assets;
mod backend;
mod tray;

use crate::backend::BackendClient;
use crate::tray::TrayManager;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tracing::{error, info};

enum UserEvent {
    StatusUpdate(String, String),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Wazuh Agent Status Rust Client...");

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let tray_manager = TrayManager::new().map_err(|e| {
        error!("Tray initialization failed: {}", e);
        e
    })?;

    // Spawn backend monitoring task
    tokio::spawn(async move {
        let backend = BackendClient::new("localhost:50505");
        if let Err(e) = backend
            .monitor_status(move |status, connection| {
                let _ = proxy.send_event(UserEvent::StatusUpdate(status, connection));
            })
            .await
        {
            error!("Backend monitoring task failed: {}", e);
        }
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let tao::event::Event::MainEventsCleared = event {
            // Standard event loop maintenance
        }

        if let tao::event::Event::UserEvent(UserEvent::StatusUpdate(status, connection)) = event {
            tray_manager.update_status(status, connection);
        }
    });
}
