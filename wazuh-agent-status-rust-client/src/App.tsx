import { useState, useEffect, type CSSProperties } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import type { AppConfig, View } from "./types/app";
import type { AgentStatus, SystemMetrics, UpdateStatus } from "./types/agent";

import { IconHome, IconShield, IconSettings } from "./components/Icons";
import { StatusView } from "./components/StatusView";
import { UpdatesView } from "./components/UpdatesView";
import { SettingsView } from "./components/SettingsView";

// ─── Defaults ─────────────────────────────────────────────────────────────────

const DEFAULT_STATUS: AgentStatus = {
  status: "Unknown",
  connection: "Disconnected",
  version: "Unknown",
  tray_version: "Unknown",
  groups: [],
};

const DEFAULT_METRICS: SystemMetrics = {
  cpu_usage: 0,
  memory_usage: 0,
  total_memory: 0,
  used_memory: 0,
  agent_running: false,
};

// ─── Loading State ────────────────────────────────────────────────────────────

function AppLoading() {
  return (
    <div className="view-container" style={{ padding: '20px' }}>
      <div className="skeleton skeleton-text" style={{ width: '80px' }} />
      <div className="skeleton skeleton-title" />
      <div className="card skeleton" style={{ height: '80px' }} />
      <div className="card skeleton" style={{ height: '80px' }} />
      <div className="card skeleton" style={{ height: '80px' }} />
    </div>
  );
}

// ─── App ──────────────────────────────────────────────────────────────────────

const STATUS_POLL_MS = 2_000;
const STORAGE_KEY_VIEW = "wazuh_active_view";

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [agentStatus, setAgentStatus] = useState<AgentStatus>(DEFAULT_STATUS);
  const [metrics, setMetrics] = useState<SystemMetrics>(DEFAULT_METRICS);
  const [updateInfo, setUpdateInfo] = useState<UpdateStatus | null>(null);
  const [activeView, setActiveView] = useState<View>(() => {
    return (localStorage.getItem(STORAGE_KEY_VIEW) as View) || "status";
  });

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY_VIEW, activeView);
  }, [activeView]);

  useEffect(() => {
    // Initial data fetch
    invoke<AppConfig>("get_config").then(setConfig).catch(console.error);
    invoke<UpdateStatus>("check_for_updates").then(setUpdateInfo).catch(console.error);
    
    // Polling logic for real-time data
    const refreshData = () => {
      invoke<AgentStatus>("get_agent_status").then(setAgentStatus).catch(console.error);
      invoke<SystemMetrics>("get_system_metrics").then(setMetrics).catch(console.error);
    };

    refreshData();
    const statusTimer = setInterval(refreshData, STATUS_POLL_MS);

    return () => clearInterval(statusTimer);
  }, []);

  useEffect(() => {
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
    };
    document.addEventListener("contextmenu", handleContextMenu);
    return () => document.removeEventListener("contextmenu", handleContextMenu);
  }, []);

  if (!config) {
    return (
      <div className="app-wrapper loading">
        <nav className="sidebar" />
        <main className="main-content">
          <AppLoading />
        </main>
      </div>
    );
  }

  const primaryColor = config.brand.theme.primary_color;
  const cssVars = { 
    "--primary": primaryColor,
    "--primary-glow": `${primaryColor}99`,
    "--primary-metallic": `linear-gradient(135deg, ${primaryColor}, #ffffff44, ${primaryColor})` 
  } as CSSProperties;

  const indicatorTop = (
    { status: "10px", updates: "70px", settings: "130px" } as Record<View, string>
  )[activeView];

  return (
    <div className="app-wrapper" style={cssVars}>
      <nav className="sidebar">
        <div className="sidebar-logo">
          <img src="/adorsys-logo.png" alt="Adorsys" />
        </div>

        <div className="nav-items" style={{ position: 'relative', display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          <div 
            className="nav-indicator" 
            style={{ top: indicatorTop }} 
          />
          
          <div className="tooltip-container">
            <button
              className={`nav-item ${activeView === "status" ? "active" : ""}`}
              onClick={() => setActiveView("status")}
              aria-label="Overview"
            >
              <IconHome />
            </button>
            <span className="tooltip">Overview</span>
          </div>

          <div className="tooltip-container">
            <button
              className={`nav-item ${activeView === "updates" ? "active" : ""}`}
              onClick={() => setActiveView("updates")}
              aria-label="Health & Updates"
            >
              <IconShield />
              {updateInfo?.has_updates && <span className="notification-dot" />}
            </button>
            <span className="tooltip">Health & Updates</span>
          </div>

          <div className="tooltip-container">
            <button
              className={`nav-item ${activeView === "settings" ? "active" : ""}`}
              onClick={() => setActiveView("settings")}
              aria-label="Settings"
            >
              <IconSettings />
            </button>
            <span className="tooltip">Settings</span>
          </div>
        </div>

        <div className="sidebar-bottom">
          <div className="copyright">© {new Date().getFullYear()} {config.brand.company}</div>
        </div>
      </nav>

      <main className="main-content">
        {activeView === "status" && <StatusView agentStatus={agentStatus} metrics={metrics} />}
        {activeView === "updates" && <UpdatesView updateInfo={updateInfo} />}
        {activeView === "settings" && <SettingsView config={config} agentStatus={agentStatus} />}
      </main>
    </div>
  );
}

export default App;
