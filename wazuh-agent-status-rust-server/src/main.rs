//! Entry point for the Wazuh Agent Status Rust server.
//!
//! Run with `--version` / `-v` to print the build version and exit.
//! All other configuration is via environment variables (see [`crate::config::Config`]).

use std::sync::Arc;

use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use wazuh_agent_status_rust_server::config::{AgentPaths, Config};
use wazuh_agent_status_rust_server::manager::AgentManager;
use wazuh_agent_status_rust_server::server::TcpServer;

#[cfg(target_os = "windows")]
use windows_service::{
    define_windows_service, service_control_handler, service_dispatcher,
    service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceStatus, ServiceType},
};

#[cfg(target_os = "windows")]
use tokio::sync::oneshot;

#[cfg(target_os = "windows")]
define_windows_service!(ffi_service_main, windows_service_main);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── CLI: --version / -v ───────────────────────────────────────────────────
    if std::env::args().any(|a| a == "--version" || a == "-v") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // ── Windows Service Check ────────────────────────────────────────────────
    #[cfg(target_os = "windows")]
    {
        // Try to run as a service. If it fails (e.g. run from console),
        // we fall back to the normal main loop below.
        if let Err(e) = service_dispatcher::start("WazuhAgentStatus", ffi_service_main) {
            // Check if error is "not running in a service context" (1063)
            if !e.to_string().contains("1063") {
                error!("Windows service dispatcher failed: {:?}", e);
            }
        } else {
            // If service_dispatcher::start succeeds, it blocks until service stops.
            return Ok(());
        }
    }

    // ── Normal Main Execution (Console) ──────────────────────────────────────
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    
    // Spawn a task to handle Ctrl+C for console mode
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            let _ = tx.send(());
        }
    });

    run_server(rx).await
}

async fn run_server(mut shutdown_rx: tokio::sync::oneshot::Receiver<()>) -> anyhow::Result<()> {
    // ── Logging setup ─────────────────────────────────────────────────────────
    let log_file = AgentPaths::log_file_path();
    let log_dir  = log_file.parent().unwrap_or(std::path::Path::new("/tmp"));
    let log_name = log_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("wazuh-agent-status.log");

    // Create log dir if it doesn't exist (best-effort)
    let _ = std::fs::create_dir_all(log_dir);

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, log_name);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_writer(std::io::stderr))   // console
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false)) // rotating file
        .init();

    // ── Configuration ─────────────────────────────────────────────────────────
    let config = Arc::new(Config::from_env());
    let paths  = Arc::new(AgentPaths::native());

    info!(
        version     = env!("CARGO_PKG_VERSION"),
        listen_addr = %config.listen_addr,
        poll_secs   = config.poll_interval.as_secs(),
        "Starting Wazuh Agent Status Rust Server"
    );

    // ── Manager ───────────────────────────────────────────────────────────────
    let manager = Arc::new(AgentManager::new(Arc::clone(&config), Arc::clone(&paths)));

    // Background polling task
    let polling_manager = Arc::clone(&manager);
    tokio::spawn(async move {
        polling_manager.start_polling().await;
    });

    // ── TCP Server + graceful shutdown ────────────────────────────────────────
    let server = TcpServer::new(config.listen_addr.clone(), Arc::clone(&manager));

    tokio::select! {
        res = server.start() => {
            if let Err(e) = res {
                error!(error = %e, "Server error");
            }
        }
        _ = &mut shutdown_rx => {
            info!("Shutdown signal received");
        }
    }

    info!("Server shutdown complete");
    Ok(())
}

#[cfg(target_os = "windows")]
fn windows_service_main(_arguments: Vec<std::ffi::OsString>) {
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let tx_arc = Arc::new(std::sync::Mutex::new(Some(tx)));

    let event_handler = move |control_event| -> service_control_handler::ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                if let Ok(mut tx_opt) = tx_arc.lock() {
                    if let Some(tx) = tx_opt.take() {
                        let _ = tx.send(());
                    }
                }
                service_control_handler::ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => service_control_handler::ServiceControlHandlerResult::NoError,
            _ => service_control_handler::ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = match service_control_handler::register("WazuhAgentStatus", event_handler) {
        Ok(h) => h,
        Err(_) => return,
    };

    let _ = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: windows_service::service::ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    });

    // Run the server in the main thread (blocking until shutdown)
    let rt = tokio::runtime::Runtime::new().unwrap();
    if let Err(e) = rt.block_on(run_server(rx)) {
        error!("Windows Service error: {:?}", e);
    }

    let _ = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: windows_service::service::ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    });
}
