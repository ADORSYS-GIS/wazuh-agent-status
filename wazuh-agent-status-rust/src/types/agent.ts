export interface AgentStatus {
    status: string;
    connection: string;
    agent_version: string;
}

export interface SystemMetrics {
    cpu_usage: number;
    memory_usage: number;
    agent_running: boolean;
}
