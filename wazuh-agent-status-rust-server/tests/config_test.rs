use std::time::Duration;
use wazuh_agent_status_rust_server::config::Config;

#[test]
fn test_default_config() {
    let cfg = Config::default();
    assert_eq!(cfg.listen_addr, "0.0.0.0:50505");
    assert_eq!(cfg.poll_interval, Duration::from_secs(5));
}

#[test]
fn test_env_override() {
    unsafe {
        std::env::set_var("WAZUH_STATUS_ADDR", "127.0.0.1:1234");
        std::env::set_var("WAZUH_STATUS_POLL_INTERVAL_SECS", "10");
    }
    
    let cfg = Config::from_env();
    assert_eq!(cfg.listen_addr, "127.0.0.1:1234");
    assert_eq!(cfg.poll_interval, Duration::from_secs(10));
    
    // Cleanup
    unsafe {
        std::env::remove_var("WAZUH_STATUS_ADDR");
        std::env::remove_var("WAZUH_STATUS_POLL_INTERVAL_SECS");
    }
}

#[test]
fn test_invalid_env_fallback() {
    unsafe {
        std::env::set_var("WAZUH_STATUS_POLL_INTERVAL_SECS", "not-a-number");
    }
    let cfg = Config::from_env();
    assert_eq!(cfg.poll_interval, Duration::from_secs(5)); // Should fall back to default
    unsafe {
        std::env::remove_var("WAZUH_STATUS_POLL_INTERVAL_SECS");
    }
}
