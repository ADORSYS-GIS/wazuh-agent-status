use wazuh_agent_status_rust_server::models::{AgentState, AgentStatus, ConnectionStatus};

#[test]
fn test_agent_state_serialization() {
    let state = AgentState {
        status: AgentStatus::Active,
        connection: ConnectionStatus::Connected,
        version: "4.7.2".to_string(),
        groups: vec!["default".to_string(), "linux".to_string()],
    };

    let json = serde_json::to_string(&state).expect("Failed to serialize");
    assert!(json.contains("\"status\":\"Active\""));
    assert!(json.contains("\"connection\":\"Connected\""));
    assert!(json.contains("\"version\":\"4.7.2\""));
}

#[test]
fn test_agent_state_deserialization() {
    let json = r#"{
        "status": "Inactive",
        "connection": "Disconnected",
        "version": "4.6.0",
        "groups": ["test"]
    }"#;

    let state: AgentState = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(state.status, AgentStatus::Inactive);
    assert_eq!(state.connection, ConnectionStatus::Disconnected);
    assert_eq!(state.version, "4.6.0");
}
