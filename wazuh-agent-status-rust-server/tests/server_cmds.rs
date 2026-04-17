use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, TcpListener};
use wazuh_agent_status_rust_server::config::{AgentPaths, Config};
use wazuh_agent_status_rust_server::manager::AgentManager;
use wazuh_agent_status_rust_server::server::TcpServer;

#[tokio::test]
async fn test_server_invalid_command() {
    let (_manager, addr) = setup_test_server().await;
    
    let stream = TcpStream::connect(&addr).await.unwrap();
    let (mut reader_raw, mut writer) = tokio::io::split(stream);
    
    writer.write_all(b"UNKNOWN_COMMAND\n").await.unwrap();
    
    let mut reader = BufReader::new(&mut reader_raw).lines();
    let line = reader.next_line().await.unwrap().unwrap();
    assert_eq!(line, "ERROR: Unknown command: UNKNOWN_COMMAND");
}

#[tokio::test]
async fn test_server_large_payload() {
    let (_manager, addr) = setup_test_server().await;
    
    let mut stream = TcpStream::connect(&addr).await.unwrap();
    // Send 2KB of data (exceeds 1024 limit)
    let large_data = vec![b'A'; 2048];
    let _ = stream.write_all(&large_data).await;
    let _ = stream.write_all(b"\n").await;
    
    // Server should close connection or error out
    let mut reader = BufReader::new(&mut stream).lines();
    let res = reader.next_line().await;
    match res {
        Ok(Some(line)) => assert!(line.contains("ERROR")),
        _ => {} // Connection closed or error is fine for large payload
    }
}

#[tokio::test]
async fn test_server_concurrent_clients() {
    let (_manager, addr) = setup_test_server().await;
    
    let mut handles = vec![];
    for _ in 0..10 { // Increased to 10 for better stress
        let addr_clone = addr.clone();
        handles.push(tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr_clone).await.expect("Failed to connect");
            stream.write_all(b"get-version\n").await.unwrap();
            let mut reader = BufReader::new(&mut stream).lines();
            let line = reader.next_line().await.unwrap().unwrap();
            assert!(line.contains("Version:") || line.contains("Unknown"));
        }));
    }
    
    for h in handles {
        h.await.unwrap();
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn setup_test_server() -> (Arc<AgentManager>, String) {
    // Find a free port by binding to 0
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind port 0");
    let addr = listener.local_addr().expect("Failed to get local addr");
    let addr_str = addr.to_string();
    drop(listener); // Release it so the server can bind to it
    
    // Give OS a tiny moment to release
    tokio::time::sleep(Duration::from_millis(10)).await;

    let config = Arc::new(Config {
        listen_addr: addr_str.clone(),
        ..Config::default()
    });
    let paths = Arc::new(AgentPaths::native());
    let manager = Arc::new(AgentManager::new(Arc::clone(&config), Arc::clone(&paths)));
    
    let server = TcpServer::new(addr_str.clone(), Arc::clone(&manager));
    
    tokio::spawn(async move {
        let _ = server.start().await;
    });
    
    // Wait for server to start
    let mut retry = 0;
    while retry < 10 {
        if TcpStream::connect(&addr_str).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        retry += 1;
    }
    
    (manager, addr_str)
}
