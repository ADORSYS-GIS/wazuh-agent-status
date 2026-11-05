use wazuh_agent_status_rust_server::version_utils::is_version_higher;

#[test]
fn test_higher_major_version() {
    assert!(is_version_higher("5.0.0", "4.9.9"));
}

#[test]
fn test_same_version_is_not_higher() {
    assert!(!is_version_higher("4.7.2", "4.7.2"));
}

#[test]
fn test_stable_beats_prerelease_of_same_base() {
    assert!(is_version_higher("4.7.2", "4.7.2-rc1"));
}

#[test]
fn test_prerelease_does_not_beat_stable() {
    assert!(!is_version_higher("4.7.2-rc1", "4.7.2"));
}

#[test]
fn test_v_prefix_stripped() {
    assert!(is_version_higher("v5.0.0", "v4.0.0"));
}

#[test]
fn test_uneven_parts() {
    assert!(is_version_higher("4.7.2.1", "4.7.2"));
    assert!(!is_version_higher("4.7", "4.7.1"));
}

#[test]
fn test_non_numeric_graceful_handling() {
    // Should parse as 0 according to implementation (unwrap_or(0))
    assert!(!is_version_higher("4.x.2", "4.1.2"));
}

#[test]
fn test_should_show_prerelease() {
    use wazuh_agent_status_rust_server::models::{FrameworkVersion, VersionInfo};
    use wazuh_agent_status_rust_server::version_utils::should_show_prerelease;

    let empty_info = VersionInfo {
        framework: FrameworkVersion {
            version: "1.0.0".to_string(),
            prerelease_version: "1.1.0-rc1".to_string(),
        },
        prerelease_test_groups: vec![],
    };
    assert!(!should_show_prerelease(&empty_info, &["beta".to_string()]));
    assert!(!should_show_prerelease(&empty_info, &[]));

    let info_with_groups = VersionInfo {
        framework: FrameworkVersion {
            version: "1.0.0".to_string(),
            prerelease_version: "1.1.0-rc1".to_string(),
        },
        prerelease_test_groups: vec!["beta".to_string(), "staging".to_string()],
    };
    assert!(!should_show_prerelease(&info_with_groups, &[]));
    assert!(should_show_prerelease(
        &info_with_groups,
        &["BETA".to_string()]
    ));
    assert!(should_show_prerelease(
        &info_with_groups,
        &["prod".to_string(), "staging".to_string()]
    ));
    assert!(!should_show_prerelease(
        &info_with_groups,
        &["prod".to_string()]
    ));
}

#[tokio::test]
async fn test_fetch_version_unreachable() {
    use wazuh_agent_status_rust_server::version_utils::{fetch_plain_version, fetch_version_info};

    assert!(
        fetch_version_info("http://127.0.0.1:9/manifest.json")
            .await
            .is_none()
    );
    assert!(
        fetch_plain_version("http://127.0.0.1:9/version.txt")
            .await
            .is_none()
    );
}
