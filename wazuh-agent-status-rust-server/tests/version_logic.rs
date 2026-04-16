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
