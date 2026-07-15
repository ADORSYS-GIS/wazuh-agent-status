export interface AgentStatus {
    status: string;
    connection: string;
    version: string;
    tray_version: string;
    groups: string[];
    self_healing_enabled: boolean;
    agent_id: string;
    agent_name: string;
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
    tray: ComponentUpdate;
    has_updates: boolean;
}

export interface SystemMetrics {
    cpu_usage: number;
    memory_usage: number;
    total_memory: number;
    used_memory: number;
    agent_running: boolean;
    agent_found?: boolean;
    agentd_found?: boolean;
}

// ── SCA / Compliance Types ──────────────────────────────────────────────────────

export interface ComplianceCheckResult {
    check_id: number;
    title: string;
    status: string;
    mandatory: boolean;
    remediation: string;
}

export interface ComplianceCategory {
    name: string;
    status: string;
    passed_count: number;
    failed_count: number;
    untested_count: number;
    checks: ComplianceCheckResult[];
}

export interface ComplianceReport {
    agent_id: string;
    agent_name: string;
    os: string;
    score: number;
    compliance_status: string;
    total_passed_count: number;
    total_failed_count: number;
    total_untested_count: number;
    categories: ComplianceCategory[];
}

export type LogLevel = "ERROR" | "WARNING" | "INFO" | "DEBUG" | "UNKNOWN";

export interface LogLine {
    raw: string;
    level: LogLevel;
}
