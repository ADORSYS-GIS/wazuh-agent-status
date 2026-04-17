export interface AgentStatus {
    status: string;
    connection: string;
    version: string;
    groups: string[];
}

export interface SystemMetrics {
    cpu_usage: number;
    memory_usage: number;
    total_memory: number;
    used_memory: number;
    agent_running: boolean;
}
