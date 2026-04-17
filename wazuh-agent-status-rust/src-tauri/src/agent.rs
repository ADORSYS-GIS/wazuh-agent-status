#[derive(Debug, serde::Serialize)]
pub struct AgentStatus {
    pub status: String,
    pub connection: String,
    pub agent_version: String,
}

pub struct AgentManager;

impl AgentManager {
    pub fn new() -> Self {
        Self
    }

    pub fn check_status(&self) -> AgentStatus {
        AgentStatus {
            status: "Active".to_string(),
            connection: "Connected".to_string(),
            agent_version: "4.14.4".to_string(),
        }
    }
}
