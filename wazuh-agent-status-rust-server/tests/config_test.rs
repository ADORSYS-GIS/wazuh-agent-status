use std::time::Duration;
use wazuh_agent_status_rust_server::config::Config;

#[test]
fn test_default_config() {
    let cfg = Config::default();
    assert_eq!(cfg.listen_addr, "0.0.0.0:50505");
    assert_eq!(cfg.poll_interval, Duration::from_secs(5));
}

#[test]
fn test_config_env_behavior() {
    // We group environment-dependent tests into one function because 
    // std::env is process-global and tests run in parallel.
    
    // 1. Test override
    unsafe {
        std::env::set_var("WAZUH_STATUS_ADDR", "127.0.0.1:1234");
        std::env::set_var("WAZUH_STATUS_POLL_INTERVAL_SECS", "10");
    }
    let cfg = Config::from_env();
    assert_eq!(cfg.listen_addr, "127.0.0.1:1234");
    assert_eq!(cfg.poll_interval, Duration::from_secs(10));
    
    // 2. Test invalid fallback
    unsafe {
        std::env::set_var("WAZUH_STATUS_POLL_INTERVAL_SECS", "not-a-number");
    }
    let cfg2 = Config::from_env();
    assert_eq!(cfg2.poll_interval, Duration::from_secs(5));
    
    // Cleanup
    unsafe {
        std::env::remove_var("WAZUH_STATUS_ADDR");
        std::env::remove_var("WAZUH_STATUS_POLL_INTERVAL_SECS");
    }
}
