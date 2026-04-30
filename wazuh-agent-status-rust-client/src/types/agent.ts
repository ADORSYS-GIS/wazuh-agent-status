export interface AgentStatus {
    status: string;
    connection: string;
    version: string;
    tray_version: string;
    groups: string[];
    self_healing_enabled: boolean;
}

export type UpdateState = "uptodate" | "outdated" | "prereleaseavailable" | "unknown";

export interface ComponentUpdate {
    name: string;
    current_version: string;
    latest_version: string;
    state: UpdateState;
    can_update: boolean;
}

export interface UpdateStatus {
    wazuh: ComponentUpdate;
    tray: ComponentUpdate;
    has_updates: boolean;
}

export interface SystemMetrics {
    cpu_usage: number;
    memory_usage: number;
    total_memory: number;
    used_memory: number;
    agent_running: boolean;
}
