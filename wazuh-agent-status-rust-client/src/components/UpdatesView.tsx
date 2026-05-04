import type { UpdateStatus, ComponentUpdate } from "../types/agent";

interface UpdatesViewProps {
  updateInfo: UpdateStatus | null;
}

export function UpdatesView({ updateInfo }: Readonly<UpdatesViewProps>) {
  return (
    <div className="view-container">
      <div className="subtitle">Security & Versions</div>
      <h2 className="header title">Health & Updates</h2>

      <div className="section-title">Deployment Manifest</div>
      <p className="hint-text" style={{ marginBottom: "20px" }}>
        Version monitoring for the Wazuh agent and Status Agent app
      </p>

      {updateInfo ? (
        <>
          <UpdateCard 
            component={updateInfo.wazuh} 
            isAutoUpdate={true} 
            description="Wazuh security framework core services."
          />
          <UpdateCard 
            component={updateInfo.tray} 
            isAutoUpdate={false} 
            description="Tray application interface."
          />
        </>
      ) : (
        <div className="card">
          <div className="card-info">
            <div className="card-label">Status</div>
            <div className="card-value">Checking for updates...</div>
          </div>
        </div>
      )}

      <div className="section-title section-title--spaced" style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
        <span>Self-Healing</span>
        <span style={{ 
          background: 'rgba(34, 197, 94, 0.15)', 
          color: '#4ade80', 
          padding: '4px 10px', 
          borderRadius: '12px', 
          fontSize: '11px', 
          fontWeight: 700, 
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
          border: '1px solid rgba(34, 197, 94, 0.2)',
          display: 'inline-flex',
          alignItems: 'center'
        }}>
          Active
        </span>
      </div>
      <p className="hint-text">
        Critical services are monitored for health and will automatically restart if they fail.
      </p>
    </div>
  );
}

interface UpdateCardProps {
  component: ComponentUpdate;
  isAutoUpdate: boolean;
  description: string;
}

function UpdateCard({ component, isAutoUpdate, description }: Readonly<UpdateCardProps>) {
  const isOutdated = component.state === "outdated" || component.state === "prereleaseavailable";

  return (
    <div className="card" style={{ flexDirection: "column", alignItems: "flex-start", gap: "10px", height: "auto", minHeight: "110px", padding: "18px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", width: "100%", alignItems: "center" }}>
        <div className="card-info">
          <div className="card-label">{component.name}</div>
          <div className="card-value" style={{ color: isOutdated ? "var(--warning)" : "var(--success)" }}>
            {isOutdated ? `Update Available (v${component.latest_version})` : `Up to date (v${component.current_version})`}
          </div>
        </div>
        {isOutdated && !isAutoUpdate && (
          <button className="update-button">Update Now</button>
        )}
        {isAutoUpdate && isOutdated && (
          <div className="auto-badge">Auto-updating...</div>
        )}
      </div>
      <p className="card-sub" style={{ margin: 0 }}>{description}</p>
      {isAutoUpdate && (
        <p className="hint-text" style={{ padding: 0, marginTop: "4px", fontSize: "0.7rem", opacity: 0.8 }}>
          TODO: Implement server-side Agent auto-orchestration.
        </p>
      )}
    </div>
  );
}
