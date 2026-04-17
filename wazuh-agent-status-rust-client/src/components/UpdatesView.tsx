interface UpdatesViewProps {
  updateInfo: string | null;
}

export function UpdatesView({ updateInfo }: Readonly<UpdatesViewProps>) {
  return (
    <div className="view-container">
      <div className="subtitle">Version Control</div>
      <h2 className="header title">Updates & Healing</h2>

      <div className="card">
        <div className="card-info">
          <div className="card-label">App Status</div>
          <div className="card-value">{updateInfo ?? "Checking..."}</div>
          <p className="card-sub">Wazuh deployment manifest versioning</p>
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
