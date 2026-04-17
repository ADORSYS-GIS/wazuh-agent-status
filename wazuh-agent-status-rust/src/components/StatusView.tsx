import type { AgentStatus, SystemMetrics } from "../types/agent";

interface StatusViewProps {
  agentStatus: AgentStatus;
  metrics: SystemMetrics;
}

export function StatusView({ agentStatus, metrics }: Readonly<StatusViewProps>) {
  return (
    <div className="view-container">
      <div className="subtitle">Real-time Activity</div>
      <h2 className="header title">Agent Deployment</h2>

      <section>
        <div className="section-title">Status Overview</div>

        <div className="card">
          <div className={`status-dot ${agentStatus.status === "Active" ? "success" : "error"}`} />
          <div className="card-info">
            <div className="card-label">Wazuh Service</div>
            <div className="card-value">{agentStatus.status}</div>
          </div>
        </div>

        <div className="card">
          <div className={`status-dot ${agentStatus.connection === "Connected" ? "success" : "error"}`} />
          <div className="card-info">
            <div className="card-label">Manager Connection</div>
            <div className="card-value">{agentStatus.connection}</div>
          </div>
        </div>

        <div className="card">
          <div className="card-info">
            <div className="card-label">Agent Version</div>
            <div className="card-value">{agentStatus.agent_version}</div>
          </div>
        </div>
      </section>

      {metrics.agent_running && (
        <section className="metrics-section">
          <div className="section-title">Agent Performance</div>
          <div className="metrics-row">
            <div className="metric-box">
              <div className="metric-label">
                <span>Agent CPU</span>
                <span>{metrics.cpu_usage.toFixed(1)}%</span>
              </div>
              <div className="progress-track">
                <div className="progress-thumb" style={{ width: `${Math.min(metrics.cpu_usage, 100)}%` }} />
              </div>
            </div>
            <div className="metric-box">
              <div className="metric-label">
                <span>Agent RAM</span>
                <span>{metrics.memory_usage.toFixed(2)}%</span>
              </div>
              <div className="progress-track">
                <div className="progress-thumb" style={{ width: `${Math.min(metrics.memory_usage, 100)}%` }} />
              </div>
            </div>
          </div>
        </section>
      )}
    </div>
  );
}
