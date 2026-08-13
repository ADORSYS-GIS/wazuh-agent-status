use std::sync::Arc;

use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use wazuh_agent_status_rust_server::config::{AgentPaths, Config};
use wazuh_agent_status_rust_server::manager::AgentManager;
use wazuh_agent_status_rust_server::server::TcpServer;

#[cfg(target_os = "windows")]
use windows_service::{
    define_windows_service,
    service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceStatus, ServiceType},
    service_control_handler, service_dispatcher,
};

#[cfg(target_os = "windows")]
define_windows_service!(ffi_service_main, windows_service_main);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" | "-h" => {
                println!("Wazuh Agent Status Server v{}", env!("CARGO_PKG_VERSION"));
                println!("Usage: wazuh-agent-status-rust-server [OPTIONS]");
                println!("  -v, --version    Print version");
                println!("  -h, --help       Print this help message");
                return Ok(());
            }
            unknown => {
                anyhow::bail!("Unknown argument: '{}'. Use --help.", unknown);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Err(e) = service_dispatcher::start("WazuhAgentStatus", ffi_service_main) {
            if !e.to_string().contains("1063") {
                error!("Windows service dispatcher failed: {:?}", e);
            }
        } else {
            return Ok(());
        }
    }

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            let _ = tx.send(());
        }
    });

    run_server(rx).await
}

async fn run_server(mut shutdown_rx: tokio::sync::oneshot::Receiver<()>) -> anyhow::Result<()> {
    let log_file = AgentPaths::log_file_path();
    let log_dir = log_file.parent().unwrap_or(std::path::Path::new("/tmp"));
    let log_name = log_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("wazuh-agent-status.log");

    let _ = std::fs::create_dir_all(log_dir);

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, log_name);
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Default to debug when WAZUH_AGENT_STATUS_DEBUG is truthy so the server-side
    // update trace is visible without also having to configure RUST_LOG.
    // Mirrors the shell is_debug() semantics (falsy values like "0" stay info).
    let default_filter = match std::env::var("WAZUH_AGENT_STATUS_DEBUG") {
        Ok(v)
            if matches!(
                v.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on" | "debug"
            ) =>
        {
            "debug"
        }
        _ => "info",
    };

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter)),
        )
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    let config = Arc::new(Config::from_env());
    let paths = Arc::new(AgentPaths::native());

    info!(
        version = env!("CARGO_PKG_VERSION"),
        listen_addr = %config.listen_addr,
        poll_secs = config.poll_interval.as_secs(),
        "Starting Wazuh Agent Status Rust Server"
    );

    let manager = Arc::new(AgentManager::new(Arc::clone(&config), Arc::clone(&paths)));

    let polling_manager = Arc::clone(&manager);
    tokio::spawn(async move {
        polling_manager.start_polling().await;
    });

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

    let event_handler =
        move |control_event| -> service_control_handler::ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Stop | ServiceControl::Shutdown => {
                    if let Ok(mut tx_opt) = tx_arc.lock() {
                        if let Some(tx) = tx_opt.take() {
                            let _ = tx.send(());
                        }
                    }
                    service_control_handler::ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => {
                    service_control_handler::ServiceControlHandlerResult::NoError
                }
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
