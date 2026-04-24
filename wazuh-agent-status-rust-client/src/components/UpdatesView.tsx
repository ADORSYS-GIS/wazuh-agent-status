import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UpdateStatus, ComponentUpdate, AgentStatus } from "../types/agent";

interface UpdatesViewProps {
  updateInfo: UpdateStatus | null;
  agentStatus: AgentStatus;
}

export function UpdatesView({ updateInfo, agentStatus }: Readonly<UpdatesViewProps>) {
  const [isUpdating, setIsUpdating] = useState(false);
  const [logs, setLogs] = useState<{ id: string; text: string }[]>([]);
  const [updateStatus, setUpdateStatus] = useState<"idle" | "running" | "success" | "error">("idle");
  const logEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unlisten = listen<string>("update-log", (event) => {
      setLogs((prev) => [...prev, { id: crypto.randomUUID(), text: event.payload }]);
      if (event.payload.includes("[SUCCESS]")) setUpdateStatus("success");
      if (event.payload.includes("[FAILURE]")) setUpdateStatus("error");
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  const handleUpdate = async (isPrerelease: boolean) => {
    setLogs([{ id: crypto.randomUUID(), text: "Starting orchestrated update..." }]);
    setIsUpdating(true);
    setUpdateStatus("running");
    try {
      await invoke("start_update", { isPrerelease });
    } catch (error) {
      setLogs((prev) => [...prev, { id: crypto.randomUUID(), text: `[ERROR] Failed to start update: ${error}` }]);
      setUpdateStatus("error");
    }
  };

  const getUpdateStatusColor = () => {
    if (updateStatus === "success") return "var(--success)";
    if (updateStatus === "error") return "var(--warning)";
    return "var(--accent)";
  };
  const statusColor = getUpdateStatusColor();

  return (
    <div className="view-container">
      <div className="subtitle">Security & Versions</div>
      <h2 className="header title">Health & Updates</h2>

      {isUpdating && (
        <div className="card update-overlay" style={{ background: "var(--bg)", border: "1px solid var(--border)", marginBottom: "20px" }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
            <span style={{ fontWeight: 600 }}>Update in Progress</span>
            <span style={{ color: statusColor }}>
              {updateStatus.toUpperCase()}
            </span>
          </div>
          <div className="log-container" style={{ 
            background: "#000", 
            padding: "10px", 
            borderRadius: "6px", 
            fontSize: "11px", 
            fontFamily: "monospace", 
            maxHeight: "150px", 
            overflowY: "auto",
            border: "1px solid #333"
          }}>
            {logs.map((log) => (
              <div key={log.id} style={{ color: log.text.includes("[ERROR]") || log.text.includes("[FAILURE]") ? "#f87171" : "#d1d5db" }}>
                {log.text}
              </div>
            ))}
            <div ref={logEndRef} />
          </div>
          {updateStatus !== "running" && (
            <button 
              className="update-button" 
              style={{ marginTop: "12px", width: "100%" }}
              onClick={() => { setIsUpdating(false); setLogs([]); setUpdateStatus("idle"); }}
            >
              Dismiss
            </button>
          )}
        </div>
      )}

      <div className="section-title">Deployment Manifest</div>
      <p className="hint-text" style={{ marginBottom: "20px" }}>
        Version monitoring for the Wazuh agent and Status Agent app
      </p>

      {updateInfo ? (
        <>
          <UpdateCard 
            component={updateInfo.wazuh} 
            description="Wazuh security framework core services."
            onUpdate={() => {}} 
            isBusy={isUpdating}
            readOnly={true}
          />
          <UpdateCard 
            component={updateInfo.tray} 
            description="Unified Status Agent orchestrator. Handles global system updates."
            onUpdate={() => handleUpdate(updateInfo.wazuh.state === "prereleaseavailable" || updateInfo.tray.state === "prereleaseavailable")}
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
      <div style={{ display: "flex", justifyContent: "space-between", width: "100%", alignItems: "center" }}>
        <div className="card-info">
          <div className="card-label">{component.name}</div>
          <div className="card-value" style={{ color: isOutdated ? "var(--warning)" : "var(--success)" }}>
            {isOutdated ? `Update Available (v${component.latest_version})` : `Version: v${component.current_version}`}
          </div>
        </div>
        {isOutdated && !isBusy && (
          <button className="update-button" onClick={onUpdate}>Update Now</button>
        )}
        {isBusy && isOutdated && (
          <div className="auto-badge">Processing...</div>
        )}
      </div>
      <p className="card-sub" style={{ margin: 0 }}>{description}</p>
    </div>
  );
}
