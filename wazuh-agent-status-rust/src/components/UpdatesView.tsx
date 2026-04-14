import type { UpdateInfo } from "../types/app";

interface UpdatesViewProps {
  updateInfo: UpdateInfo | null;
}

export function UpdatesView({ updateInfo }: UpdatesViewProps) {
  return (
    <div className="view-container">
      <div className="subtitle">Version Control</div>
      <h2 className="header title">Updates & Healing</h2>

      <div className="card">
        <div className="card-info">
          <div className="card-label">App Status</div>
          <div className="card-value">Up to date</div>
          <p className="card-sub">Version {updateInfo?.current_version ?? "—"}</p>
        </div>
      </div>

      <div className="section-title section-title--spaced">Service Persistence</div>
      <div className="card">
        <div className="card-info">
          <div className="card-label">Self-Healing</div>
          <div className="card-value">Enabled</div>
        </div>
      </div>
      <p className="hint-text">
        The agent is configured to automatically recover from service failures.
      </p>
    </div>
  );
}
