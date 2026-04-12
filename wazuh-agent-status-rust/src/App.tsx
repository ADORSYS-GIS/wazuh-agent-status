import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface AppConfig {
  brand: {
    name: string;
    company: string;
    theme: {
      primary_color: string;
    };
  };
}

interface AgentStatus {
  status: string;
  connection: string;
  agent_version: string;
}

interface SystemMetrics {
  cpu_usage: number;
  memory_usage: number;
  agent_running: boolean;
}

interface UpdateResult {
  current_version: string;
  latest_version: string;
  update_available: boolean;
  download_url: string;
}

type View = "status" | "updates" | "settings";

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [agentStatus, setAgentStatus] = useState<AgentStatus>({ status: "Unknown", connection: "Disconnected", agent_version: "Unknown" });
  const [metrics, setMetrics] = useState<SystemMetrics>({ cpu_usage: 0, memory_usage: 0, agent_running: false });
  const [updateInfo, setUpdateInfo] = useState<UpdateResult | null>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);
  const [activeView, setActiveView] = useState<View>("status");

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig).catch(console.error);
    invoke<UpdateResult>("check_for_update").then(setUpdateInfo).catch(console.error);

    const statusInterval = setInterval(async () => {
      try {
        const res = await invoke<AgentStatus>("get_agent_status");
        setAgentStatus(res);
      } catch (err) { console.error(err); }
    }, 2000);

    const metricsInterval = setInterval(async () => {
      try {
        const res = await invoke<SystemMetrics>("get_system_metrics");
        setMetrics(res);
      } catch (err) { console.error(err); }
    }, 10000);

    return () => {
      clearInterval(statusInterval);
      clearInterval(metricsInterval);
    };
  }, []);

  const handleUpdate = async () => {
    if (!updateInfo) return;
    setIsUpdating(true);
    setUpdateError(null);
    try {
      await invoke("perform_update", { downloadUrl: updateInfo.download_url });
      // App restarts on success
    } catch (err: any) {
      setUpdateError(err.toString());
      setIsUpdating(false);
    }
  };

  if (!config) return <div className="app-wrapper">Loading...</div>;

  const handleMinimize = () => invoke("minimize_window").catch(console.error);
  const handleClose = () => invoke("hide_window").catch(console.error);

  const renderView = () => {
    switch (activeView) {
      case "status":
        return (
          <div className="view-container">
            <div className="subtitle">Real-time Activity</div>
            <h2 className="header title">Agent Deployment</h2>
            <section>
              <div className="section-title">Status Overview</div>
              <div className="card">
                <div className={`status-dot ${agentStatus.status === "Active" ? "success" : "error"}`}></div>
                <div className="card-info">
                  <div className="card-label">Wazuh Service</div>
                  <div className="card-value">{agentStatus.status}</div>
                </div>
              </div>
              <div className="card">
                <div className={`status-dot ${agentStatus.connection === "Connected" ? "success" : "error"}`}></div>
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
              <section style={{ marginTop: 'auto' }}>
                <div className="section-title">Agent Performance</div>
                <div className="metrics-row">
                  <div className="metric-box">
                    <div className="metric-label"><span>Agent CPU</span><span>{metrics.cpu_usage.toFixed(1)}%</span></div>
                    <div className="progress-track"><div className="progress-thumb" style={{ width: `${Math.min(metrics.cpu_usage, 100)}%` }}></div></div>
                  </div>
                  <div className="metric-box">
                    <div className="metric-label"><span>Agent RAM</span><span>{metrics.memory_usage.toFixed(2)}%</span></div>
                    <div className="progress-track"><div className="progress-thumb" style={{ width: `${Math.min(metrics.memory_usage, 100)}%` }}></div></div>
                  </div>
                </div>
              </section>
            )}
          </div>
        );
      case "updates":
        return (
          <div className="view-container">
            <div className="subtitle">Version Control</div>
            <h2 className="header title">Updates & Healing</h2>
            {updateInfo?.update_available ? (
              <div className="card" style={{ border: '1px solid var(--primary)', background: 'rgba(0, 170, 255, 0.05)' }}>
                <div className="card-info">
                  <div className="card-label" style={{ color: 'var(--primary)' }}>New Version Available!</div>
                  <div className="card-value">{updateInfo.latest_version}</div>
                  <p style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginTop: '8px' }}>Your version: {updateInfo.current_version}</p>
                  <button className="action-button" disabled={isUpdating} style={{ marginTop: '16px', width: '100%', background: 'var(--primary)', color: 'white', border: 'none', borderRadius: '8px', padding: '10px', fontWeight: '600', cursor: isUpdating ? 'wait' : 'pointer', opacity: isUpdating ? 0.7 : 1 }} onClick={handleUpdate}>
                    {isUpdating ? "Installing Update..." : "Install Automatically"}
                  </button>
                  {updateError && <p style={{ color: '#ff4444', fontSize: '0.75rem', marginTop: '8px' }}>Error: {updateError}</p>}
                </div>
              </div>
            ) : (
              <div className="card">
                <div className="card-info">
                  <div className="card-label">App Status</div>
                  <div className="card-value">Up to date</div>
                  <p style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginTop: '8px' }}>Version {updateInfo?.current_version || "Checking..."}</p>
                </div>
              </div>
            )}
            <div className="section-title" style={{ marginTop: '24px' }}>Service Persistence</div>
            <div className="card"><div className="card-info"><div className="card-label">Self-Healing</div><div className="card-value">Enabled</div></div></div>
            <p style={{ fontSize: '0.8rem', color: 'var(--text-dim)', padding: '0 8px', marginTop: '12px' }}>The agent is configured to automatically recover from service failures.</p>
          </div>
        );
      case "settings":
        return (
          <div className="view-container">
            <div className="subtitle">Branding</div>
            <h2 className="header title">App Settings</h2>
            <div className="card"><div className="card-info"><div className="card-label">Managed By</div><div className="card-value">{config.brand.company}</div></div></div>
            <div className="card"><div className="card-info"><div className="card-label">Environment</div><div className="card-value">Production</div></div></div>
          </div>
        );
    }
  };

  return (
    <>
      <header className="titlebar">
        <div data-tauri-drag-region className="titlebar-drag-region"></div>
        <div className="titlebar-content">
          <span className="titlebar-title">{config.brand.name}</span>
        </div>
        <div className="titlebar-actions">
          <button className="titlebar-button" id="titlebar-minimize" onClick={handleMinimize} aria-label="Minimize">
            <svg width="12" height="12" viewBox="0 0 12 12"><rect fill="currentColor" x="2" y="5.5" width="8" height="1"/></svg>
          </button>
          <button className="titlebar-button close" id="titlebar-close" onClick={handleClose} aria-label="Close">
            <svg width="12" height="12" viewBox="0 0 12 12"><path fill="currentColor" d="M11 1.576L10.424 1 6 5.424 1.576 1 1 1.576 5.424 6 1 10.424l.576.576L6 6.576 10.424 11l.576-.576L6.576 6z"/></svg>
          </button>
        </div>
      </header>
      <div className="app-wrapper" style={{ "--primary": config.brand.theme.primary_color } as any}>
        <nav className="sidebar">
          <div className={`nav-item ${activeView === "status" ? "active" : ""}`} onClick={() => setActiveView("status")} title="Overview">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>
          </div>
          <div className={`nav-item ${activeView === "updates" ? "active" : ""}`} onClick={() => setActiveView("updates")} title="Healing" style={{ position: 'relative' }}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path></svg>
            {updateInfo?.update_available && (
              <div className="status-dot error" style={{
                position: 'absolute',
                top: '6px',
                right: '6px',
                width: '11px',
                height: '11px',
                border: '2px solid var(--sidebar-bg)',
              }} />
            )}
          </div>
          <div className={`nav-item ${activeView === "settings" ? "active" : ""}`} onClick={() => setActiveView("settings")} title="Settings">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1-1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
          </div>
          <div className="sidebar-bottom">
            <div className="copyright">© {new Date().getFullYear()} {config.brand.company}</div>
          </div>
        </nav>
        <main className="main-content">{renderView()}</main>
      </div>
    </>
  );
}

export default App;
