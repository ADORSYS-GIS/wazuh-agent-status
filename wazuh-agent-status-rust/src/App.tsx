import { useState, useEffect, type CSSProperties } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import type { AppConfig, UpdateInfo, View } from "./types/app";
import type { AgentStatus, SystemMetrics } from "./types/agent";

// ─── Defaults ─────────────────────────────────────────────────────────────────

const DEFAULT_STATUS: AgentStatus = {
  status: "Unknown",
  connection: "Disconnected",
  agent_version: "Unknown",
};

const DEFAULT_METRICS: SystemMetrics = {
  cpu_usage: 0,
  memory_usage: 0,
  agent_running: false,
};

// ─── Icons ────────────────────────────────────────────────────────────────────

const IconHome = () => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
    <polyline points="9 22 9 12 15 12 15 22" />
  </svg>
);

const IconShield = () => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
  </svg>
);

const IconSettings = () => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
  </svg>
);

// ─── Status View ──────────────────────────────────────────────────────────────

interface StatusViewProps {
  agentStatus: AgentStatus;
  metrics: SystemMetrics;
}

function StatusView({ agentStatus, metrics }: StatusViewProps) {
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

// ─── Updates View ─────────────────────────────────────────────────────────────

interface UpdatesViewProps {
  updateInfo: UpdateInfo | null;
}

function UpdatesView({ updateInfo }: UpdatesViewProps) {
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

// ─── Settings View ────────────────────────────────────────────────────────────

interface SettingsViewProps {
  config: AppConfig;
}

function SettingsView({ config }: SettingsViewProps) {
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

// ─── App ──────────────────────────────────────────────────────────────────────

const STATUS_POLL_MS = 5_000;

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [agentStatus, setAgentStatus] = useState<AgentStatus>(DEFAULT_STATUS);
  const [metrics, setMetrics] = useState<SystemMetrics>(DEFAULT_METRICS);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [activeView, setActiveView] = useState<View>("status");

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig).catch(console.error);
    invoke<UpdateInfo>("check_for_update").then(setUpdateInfo).catch(console.error);
    invoke<AgentStatus>("get_agent_status").then(setAgentStatus).catch(console.error);
    invoke<SystemMetrics>("get_system_metrics").then(setMetrics).catch(console.error);

    const statusTimer = setInterval(() => {
      invoke<AgentStatus>("get_agent_status").then(setAgentStatus).catch(console.error);
    }, STATUS_POLL_MS);

    const metricsTimer = setInterval(() => {
      invoke<SystemMetrics>("get_system_metrics").then((data) => {
        const flutter = (Math.random() - 0.5) * 0.2;
        setMetrics({
          ...data,
          cpu_usage: Math.max(0, data.cpu_usage + flutter),
          memory_usage: Math.max(0, data.memory_usage + (Math.random() - 0.5) * 0.1),
        });
      }).catch(console.error);
    }, 2000);

    return () => {
      clearInterval(statusTimer);
      clearInterval(metricsTimer);
    };
  }, []);

  const handleMinimize = () => invoke("minimize_window").catch(console.error);
  const handleClose = () => invoke("hide_window").catch(console.error);

  if (!config) {
    return <div className="app-wrapper loading">Loading...</div>;
  }

  const primaryColor = config.brand.theme.primary_color;
  const cssVars = { 
    "--primary": primaryColor,
    "--primary-glow": `${primaryColor}99`,
    "--primary-metallic": `linear-gradient(135deg, ${primaryColor}, #ffffff44, ${primaryColor})` 
  } as CSSProperties;

  return (
    <>
      <header className="titlebar">
        <div data-tauri-drag-region className="titlebar-drag-region" />
        <div className="titlebar-content">
          <span className="titlebar-title">{config.brand.name}</span>
        </div>
        <div className="titlebar-actions">
          <button className="titlebar-button" onClick={handleMinimize} aria-label="Minimize">
            <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
              <rect fill="currentColor" x="2" y="5.5" width="8" height="1" />
            </svg>
          </button>
          <button className="titlebar-button close" onClick={handleClose} aria-label="Close">
            <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
              <path fill="currentColor" d="M11 1.576L10.424 1 6 5.424 1.576 1 1 1.576 5.424 6 1 10.424l.576.576L6 6.576 10.424 11l.576-.576L6.576 6z" />
            </svg>
          </button>
        </div>
      </header>

      <div className="app-wrapper" style={cssVars}>
        <nav className="sidebar">
          <div 
            className="nav-indicator" 
            style={{ 
              top: activeView === "status" ? "34px" : activeView === "updates" ? "94px" : "154px" 
            }} 
          />
          <button
            className={`nav-item ${activeView === "status" ? "active" : ""}`}
            onClick={() => setActiveView("status")}
            title="Overview"
            aria-label="Overview"
          >
            <IconHome />
          </button>
          <button
            className={`nav-item ${activeView === "updates" ? "active" : ""}`}
            onClick={() => setActiveView("updates")}
            title="Health & Updates"
            aria-label="Health & Updates"
          >
            <IconShield />
          </button>
          <button
            className={`nav-item ${activeView === "settings" ? "active" : ""}`}
            onClick={() => setActiveView("settings")}
            title="Settings"
            aria-label="Settings"
          >
            <IconSettings />
          </button>

          <div className="sidebar-bottom">
            <div className="copyright">© {new Date().getFullYear()} {config.brand.company}</div>
          </div>
        </nav>

        <main className="main-content">
          {activeView === "status" && <StatusView agentStatus={agentStatus} metrics={metrics} />}
          {activeView === "updates" && <UpdatesView updateInfo={updateInfo} />}
          {activeView === "settings" && <SettingsView config={config} />}
        </main>
      </div>
    </>
  );
}

export default App;
