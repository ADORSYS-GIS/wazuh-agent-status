use wazuh_agent_status_rust_server::models::{
    AgentState, AgentStatus, ConnectionStatus, SystemMetrics,
};

#[test]
fn test_agent_state_serialization() {
    let state = AgentState {
        status: AgentStatus::Active,
        connection: ConnectionStatus::Connected,
        version: "4.7.2".to_string(),
        tray_version: "1.8.0".to_string(),
        groups: vec!["default".to_string(), "linux".to_string()],
        metrics: SystemMetrics::default(),
        self_healing_enabled: true,
        agent_id: "020".to_string(),
        agent_name: "test-agent".to_string(),
        agent_key: "ABC123KEY".to_string(),
    };

    let json = serde_json::to_string(&state).expect("Failed to serialize");
    assert!(json.contains("\"status\":\"Active\""));
    assert!(json.contains("\"connection\":\"Connected\""));
    assert!(json.contains("\"version\":\"4.7.2\""));
    assert!(json.contains("\"tray_version\":\"1.8.0\""));
    assert!(json.contains("\"metrics\""));
    assert!(json.contains("\"agent_id\":\"020\""));
    assert!(json.contains("\"agent_name\":\"test-agent\""));
    assert!(json.contains("\"agent_key\":\"ABC123KEY\""));
}

#[test]
fn test_agent_state_deserialization() {
    let json = r#"{
        "status": "Inactive",
        "connection": "Disconnected",
        "version": "4.6.0",
        "tray_version": "1.7.0",
        "groups": ["test"],
        "metrics": {
            "cpu_usage": 0.0,
            "memory_usage": 0.0,
            "total_memory": 0,
            "used_memory": 0
        },
        "self_healing_enabled": true,
        "agent_id": "020",
        "agent_name": "test-agent",
        "agent_key": "ABC123KEY"
    }"#;

    let state: AgentState = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(state.status, AgentStatus::Inactive);
    assert_eq!(state.connection, ConnectionStatus::Disconnected);
    assert_eq!(state.version, "4.6.0");
    assert_eq!(state.tray_version, "1.7.0");
    assert_eq!(state.agent_id, "020");
    assert_eq!(state.agent_name, "test-agent");
    assert_eq!(state.agent_key, "ABC123KEY");
    // agentd_found defaults to false when not present in JSON (backward compat)
    assert!(!state.metrics.agentd_found);
}

#[test]
fn test_system_metrics_agentd_found_serialization() {
    let metrics = SystemMetrics {
        cpu_usage: 5.0,
        memory_usage: 0.01,
        total_memory: 16000000000,
        used_memory: 50000000,
        agent_found: true,
        agentd_found: true,
    };
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("\"agentd_found\":true"));

    // Round-trip: deserialize and check agentd_found is preserved
    let deserialized: SystemMetrics = serde_json::from_str(&json).unwrap();
    assert!(deserialized.agentd_found);
}

#[test]
fn test_system_metrics_agentd_found_default() {
    let json = r#"{"cpu_usage": 2.0, "memory_usage": 0.5, "total_memory": 8000000000, "used_memory": 4000000000, "agent_found": true}"#;
    let metrics: SystemMetrics = serde_json::from_str(json).unwrap();
    // agentd_found should default to false when missing from JSON
    assert!(!metrics.agentd_found);
    assert!(metrics.agent_found);
}
