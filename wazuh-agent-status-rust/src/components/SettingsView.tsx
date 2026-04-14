import type { AppConfig } from "../types/app";

interface SettingsViewProps {
  config: AppConfig;
}

export function SettingsView({ config }: SettingsViewProps) {
  return (
    <div className="view-container">
      <div className="subtitle">Branding</div>
      <h2 className="header title">App Settings</h2>

      <div className="card">
        <div className="card-info">
          <div className="card-label">Managed By</div>
          <div className="card-value">{config.brand.company}</div>
        </div>
      </div>

      <div className="card">
        <div className="card-info">
          <div className="card-label">Environment</div>
          <div className="card-value">Production</div>
        </div>
      </div>
    </div>
  );
}
