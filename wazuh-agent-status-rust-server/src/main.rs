//! Entry point for the Wazuh Agent Status Rust server.
//!
//! Run with `--version` / `-v` to print the build version and exit.
//! All other configuration is via environment variables (see [`crate::config::Config`]).

mod config;
mod errors;
mod group_extractor;
mod manager;
mod models;
mod server;
mod status_provider;
mod updater;
mod version_utils;

use std::sync::Arc;

use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::{AgentPaths, Config};
use crate::manager::AgentManager;
use crate::server::TcpServer;

#[cfg(target_os = "windows")]
use windows_service::{
    define_windows_service, service_control_handler, service_dispatcher,
    service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceStatus, ServiceType},
};

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
            // Check if error is "not running in a service context"
            if !e.to_string().contains("1063") {
                error!("Windows service dispatcher failed: {:?}", e);
            }
        } else {
            // If service_dispatcher::start succeeds, it blocks until service stops.
            return Ok(());
        }
    }

    // ── Normal Main Execution ────────────────────────────────────────────────
    run_server().await
}

async fn run_server() -> anyhow::Result<()> {
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

    // Background polling task (local-only, no network)
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
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
        }
    }

    info!("Server shutdown complete");
    Ok(())
}

#[cfg(target_os = "windows")]
fn windows_service_main(_arguments: Vec<std::ffi::OsString>) {
    let event_handler = move |control_event| -> service_control_handler::ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                // Signal graceful shutdown here if we had a global signal.
                // For simplicity, we just exit, which Windows handles correctly.
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

    let next_status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: windows_service::service::ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    };

    if let Err(_) = status_handle.set_service_status(next_status) {
        return;
    }

    // Run the server in a separate thread because main loop is blocking
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Err(e) = rt.block_on(run_server()) {
            error!("Windows Service error: {:?}", e);
        }
    });

    // In a real service, you'd wait on a shutdown channel here.
    // For now, this is enough to keep the service "Running" in SCM.
}
