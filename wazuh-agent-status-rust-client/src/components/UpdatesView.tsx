import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UpdateStatus, ComponentUpdate, AgentStatus } from "../types/agent";
import { UpdateModal } from "./UpdateModal";

interface UpdatesViewProps {
  updateInfo: UpdateStatus | null;
  agentStatus: AgentStatus;
  onRefreshUpdates: () => void;
}

export function UpdatesView({ updateInfo, agentStatus, onRefreshUpdates }: Readonly<UpdatesViewProps>) {
  const [isUpdating, setIsUpdating] = useState(false);
  const [logs, setLogs] = useState<{ id: string; text: string }[]>([]);
  const [updateStatus, setUpdateStatus] = useState<"idle" | "running" | "success" | "error">("idle");

  useEffect(() => {
    const unlisten = listen<string>("update-log", (event) => {
      setLogs((prev) => [...prev, { id: crypto.randomUUID(), text: event.payload }]);
      // Only consider the update fully successful when the server sends the final "completed" message,
      // not on intermediate [SUCCESS] messages from the script (e.g. "Installation validated successfully").
      if (event.payload.includes("UPDATE_PROGRESS: [SUCCESS] Update completed successfully")) {
        setUpdateStatus("success");
        onRefreshUpdates();
      }
      if (event.payload.includes("[FAILURE]")) setUpdateStatus("error");
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [onRefreshUpdates]);

  useEffect(() => {
    if (isUpdating && updateInfo) {
      const trayUpdated = agentStatus.tray_version !== "Unknown" && agentStatus.tray_version === updateInfo.tray.latest_version;
      
      if (trayUpdated) {
        setUpdateStatus("success");
        onRefreshUpdates();
      }
    }
  }, [agentStatus.tray_version, isUpdating, updateInfo, onRefreshUpdates]);

  const handleUpdate = async (isPrerelease: boolean) => {
    setLogs([{ id: crypto.randomUUID(), text: "[STATUS] Starting orchestrated update..." }]);
    setIsUpdating(true);
    setUpdateStatus("running");
    try {
      await invoke("start_update", { isPrerelease });
    } catch (error) {
      setLogs((prev) => [...prev, { id: crypto.randomUUID(), text: `[FAILURE] Failed to start update: ${error}` }]);
      setUpdateStatus("error");
    }
  };

  const dismissUpdate = () => {
    setIsUpdating(false);
    setLogs([]);
    setUpdateStatus("idle");
    onRefreshUpdates();
  };

  return (
    <div className="view-container">
      <div className="subtitle">Security & Versions</div>
      <h2 className="header title">Health & Updates</h2>

      {isUpdating && updateInfo && (
        <UpdateModal
          status={updateStatus === "idle" ? "running" : updateStatus}
          logs={logs}
          targetVersion={updateInfo.tray.latest_version}
          onDismiss={dismissUpdate}
        />
      )}

      <div className="section-title">Deployment Manifest</div>
      <p className="hint-text" style={{ marginBottom: "20px" }}>
        Version monitoring for the Wazuh agent and Status Agent app
      </p>

      {updateInfo ? (
        <>
          <UpdateCard 
            component={updateInfo.tray} 
            description="Unified Status Agent orchestrator. Handles global system updates."
            onUpdate={() => handleUpdate(updateInfo.tray.state === "prereleaseavailable")}
            isBusy={isUpdating}
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
          background: agentStatus.self_healing_enabled ? 'rgba(34, 197, 94, 0.15)' : 'rgba(234, 88, 12, 0.15)', 
          color: agentStatus.self_healing_enabled ? '#4ade80' : '#fb923c', 
          padding: '4px 10px', 
          borderRadius: '12px', 
          fontSize: '11px', 
          fontWeight: 700, 
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
          border: agentStatus.self_healing_enabled ? '1px solid rgba(34, 197, 94, 0.2)' : '1px solid rgba(234, 88, 12, 0.2)',
          display: 'inline-flex',
          alignItems: 'center'
        }}>
          {agentStatus.self_healing_enabled ? 'Active' : 'Disabled'}
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
  description: string;
  onUpdate: () => void;
  isBusy?: boolean;
  readOnly?: boolean;
}

function UpdateCard({ component, description, onUpdate, isBusy, readOnly }: Readonly<UpdateCardProps>) {
  const isOutdated = !readOnly && (component.state === "outdated" || component.state === "prereleaseavailable");

  return (
    <div className="card" style={{ flexDirection: "column", alignItems: "flex-start", gap: "10px", height: "auto", minHeight: "110px", padding: "18px" }}>
      <div style={{ display: "flex", justifyContent: "space-between", width: "100%", alignItems: "center", flexWrap: "wrap", gap: "12px" }}>
        <div className="card-info" style={{ minWidth: "200px", flex: "1 1 auto" }}>
          <div className="card-label">{component.name}</div>
          <div className="card-value" style={{ color: isOutdated ? "var(--warning)" : "var(--success)", whiteSpace: "nowrap" }}>
            {isOutdated ? `Update Available (v${component.latest_version})` : `Version: v${component.current_version}`}
          </div>
        </div>
        {isOutdated && !isBusy && (
          <button className="update-button" style={{ flexShrink: 0 }} onClick={onUpdate}>Update Now</button>
        )}
      </div>
      <p className="card-sub" style={{ margin: 0 }}>{description}</p>
    </div>
  );
}
