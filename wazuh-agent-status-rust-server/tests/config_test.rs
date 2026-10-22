use std::time::Duration;
use wazuh_agent_status_rust_server::config::Config;

#[test]
fn test_default_config() {
    let cfg = Config::default();
    assert_eq!(cfg.listen_addr, "127.0.0.1:50505");
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
        Duration::from_secs(14400),
        "auto-update check interval should default to 4 hours"
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
            "WAZUH_STATUS_VERSION_URL",
            "https://example.com/versions.json",
        );
        std::env::set_var(
            "WAZUH_STATUS_STABLE_VERSION_URL",
            "https://example.com/version.txt",
        );
        std::env::set_var("WAZUH_STATUS_AUTO_UPDATE_CHECK_INTERVAL_SECS", "3600");
        std::env::set_var("WAZUH_STATUS_VERSION_CACHE_TTL_SECS", "7200");
        std::env::set_var("WAZUH_STATUS_MAX_CONNECTIONS", "10");
        std::env::set_var("WAZUH_STATUS_SELF_HEALING", "false");
        std::env::set_var("WAZUH_STATUS_OSSEC_LOG", "/tmp/custom_ossec.log");
    }
    let cfg = Config::from_env();
    assert_eq!(cfg.listen_addr, "127.0.0.1:1234");
    assert_eq!(cfg.poll_interval, Duration::from_secs(10));
    assert_eq!(cfg.version_url, "https://example.com/versions.json");
    assert_eq!(cfg.stable_version_url, "https://example.com/version.txt");
    assert_eq!(
        cfg.auto_update_check_interval,
        Duration::from_secs(3600),
        "auto-update interval should be overridable via env var"
    );
    assert_eq!(cfg.version_cache_ttl, Duration::from_secs(7200));
    assert_eq!(cfg.max_connections, 10);
    assert!(!cfg.self_healing);

    let paths = wazuh_agent_status_rust_server::config::AgentPaths::native();
    assert_eq!(
        paths.ossec_log,
        std::path::PathBuf::from("/tmp/custom_ossec.log")
    );

    // 2. Test self-healing boolean variations: "true", "1", "yes", "0", "no"
    for truthy in ["true", "1", "yes"] {
        unsafe {
            std::env::set_var("WAZUH_STATUS_SELF_HEALING", truthy);
        }
        assert!(Config::from_env().self_healing);
    }
    for falsy in ["false", "0", "no"] {
        unsafe {
            std::env::set_var("WAZUH_STATUS_SELF_HEALING", falsy);
        }
        assert!(!Config::from_env().self_healing);
    }

    // 3. Test invalid fallback
    unsafe {
        std::env::set_var("WAZUH_STATUS_POLL_INTERVAL_SECS", "not-a-number");
        std::env::set_var(
            "WAZUH_STATUS_AUTO_UPDATE_CHECK_INTERVAL_SECS",
            "not-a-number",
        );
        std::env::set_var("WAZUH_STATUS_VERSION_CACHE_TTL_SECS", "not-a-number");
        std::env::set_var("WAZUH_STATUS_MAX_CONNECTIONS", "not-a-number");
        std::env::set_var("WAZUH_STATUS_SELF_HEALING", "not-a-boolean");
        std::env::remove_var("WAZUH_STATUS_OSSEC_LOG");
    }
    let cfg2 = Config::from_env();
    assert_eq!(cfg2.poll_interval, Duration::from_secs(5));
    assert_eq!(
        cfg2.auto_update_check_interval,
        Duration::from_secs(14400),
        "invalid auto-update interval should fall back to default"
    );
    assert_eq!(cfg2.version_cache_ttl, Duration::from_secs(1800));
    assert_eq!(cfg2.max_connections, 3);
    assert!(cfg2.self_healing);

    let default_paths = wazuh_agent_status_rust_server::config::AgentPaths::native();
    assert!(
        default_paths
            .ossec_log
            .to_string_lossy()
            .contains("ossec.log")
    );

    // Test log_file_path default and override
    let log_path_default = wazuh_agent_status_rust_server::config::AgentPaths::log_file_path();
    assert!(log_path_default.to_string_lossy().contains("wazuh-agent-status.log"));

    unsafe {
        std::env::set_var("WAZUH_STATUS_LOG_FILE", "/tmp/custom-status.log");
    }
    let log_path_custom = wazuh_agent_status_rust_server::config::AgentPaths::log_file_path();
    assert_eq!(log_path_custom, std::path::PathBuf::from("/tmp/custom-status.log"));
    unsafe {
        std::env::remove_var("WAZUH_STATUS_LOG_FILE");
    }

    // Test non-localhost warning branch
    unsafe {
        std::env::set_var("WAZUH_STATUS_ADDR", "0.0.0.0:50505");
    }
    let cfg_public = Config::from_env();
    assert_eq!(cfg_public.listen_addr, "0.0.0.0:50505");

    // Cleanup
    unsafe {
        std::env::remove_var("WAZUH_STATUS_ADDR");
        std::env::remove_var("WAZUH_STATUS_POLL_INTERVAL_SECS");
        std::env::remove_var("WAZUH_STATUS_VERSION_URL");
        std::env::remove_var("WAZUH_STATUS_STABLE_VERSION_URL");
        std::env::remove_var("WAZUH_STATUS_AUTO_UPDATE_CHECK_INTERVAL_SECS");
        std::env::remove_var("WAZUH_STATUS_VERSION_CACHE_TTL_SECS");
        std::env::remove_var("WAZUH_STATUS_MAX_CONNECTIONS");
        std::env::remove_var("WAZUH_STATUS_SELF_HEALING");
        std::env::remove_var("WAZUH_STATUS_OSSEC_LOG");
    }
}
