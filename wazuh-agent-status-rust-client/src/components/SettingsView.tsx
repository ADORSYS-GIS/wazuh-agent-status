import type { AppConfig } from "../types/app";
import type { AgentStatus } from "../types/agent";

interface SettingsViewProps {
  config: AppConfig;
  agentStatus: AgentStatus;
}

export function SettingsView({ config, agentStatus }: Readonly<SettingsViewProps>) {
  return (
    <div className="view-container">
      <div className="subtitle">System Information</div>
      <h2 className="header title">App Settings</h2>

      <div className="card">
        <div className="card-info">
          <div className="card-label">Tray App Version</div>
          <div className="card-value">{agentStatus.tray_version}</div>
        </div>
      </div>

      <div className="card">
        <div className="card-info">
          <div className="card-label">Managed By</div>
          <div className="card-value">{config.brand.company}</div>
        </div>
      </div>

      <div className="section-title section-title--spaced">Environment</div>
      <div className="card">
        <div className="card-info">
          <div className="card-label">Status</div>
          <div className="card-value">Production</div>
        </div>
      </div>
    </div>
  );
}
