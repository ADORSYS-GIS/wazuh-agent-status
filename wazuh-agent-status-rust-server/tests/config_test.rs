use std::time::Duration;
use wazuh_agent_status_rust_server::config::Config;

#[test]
fn test_default_config() {
    let cfg = Config::default();
    assert_eq!(cfg.listen_addr, "127.0.0.1:50506");
    assert_eq!(cfg.poll_interval, Duration::from_secs(5));
    assert!(
        cfg.version_url.contains("versions.json"),
        "version_url should point to versions.json for pre-release checks"
    );
    assert!(
        cfg.stable_version_url.contains("version.txt"),
        "stable_version_url should point to the plain version.txt"
    );
    assert_eq!(
        cfg.auto_update_check_interval,
        Duration::from_secs(1800),
        "auto-update check interval should default to 30 minutes"
    );
}

#[test]
fn test_config_env_behavior() {
    // We group environment-dependent tests into one function because
    // std::env is process-global and tests run in parallel.

    // 1. Test override
    unsafe {
        std::env::set_var("WAZUH_STATUS_ADDR", "127.0.0.1:1234");
        std::env::set_var("WAZUH_STATUS_POLL_INTERVAL_SECS", "10");
        std::env::set_var(
            "WAZUH_STATUS_STABLE_VERSION_URL",
            "https://example.com/version.txt",
        );
        std::env::set_var("WAZUH_STATUS_AUTO_UPDATE_CHECK_INTERVAL_SECS", "3600");
    }
    let cfg = Config::from_env();
    assert_eq!(cfg.listen_addr, "127.0.0.1:1234");
    assert_eq!(cfg.poll_interval, Duration::from_secs(10));
    assert_eq!(cfg.stable_version_url, "https://example.com/version.txt");
    assert_eq!(
        cfg.auto_update_check_interval,
        Duration::from_secs(3600),
        "auto-update interval should be overridable via env var"
    );

    // 2. Test invalid fallback
    unsafe {
        std::env::set_var("WAZUH_STATUS_POLL_INTERVAL_SECS", "not-a-number");
        std::env::set_var(
            "WAZUH_STATUS_AUTO_UPDATE_CHECK_INTERVAL_SECS",
            "not-a-number",
        );
    }
    let cfg2 = Config::from_env();
    assert_eq!(cfg2.poll_interval, Duration::from_secs(5));
    assert_eq!(
        cfg2.auto_update_check_interval,
        Duration::from_secs(1800),
        "invalid auto-update interval should fall back to default"
    );

    // Cleanup
    unsafe {
        std::env::remove_var("WAZUH_STATUS_ADDR");
        std::env::remove_var("WAZUH_STATUS_POLL_INTERVAL_SECS");
        std::env::remove_var("WAZUH_STATUS_STABLE_VERSION_URL");
        std::env::remove_var("WAZUH_STATUS_AUTO_UPDATE_CHECK_INTERVAL_SECS");
    }
}

