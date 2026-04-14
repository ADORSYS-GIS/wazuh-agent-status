import { useState, useEffect, type CSSProperties } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import type { AppConfig, UpdateInfo, View } from "./types/app";
import type { AgentStatus, SystemMetrics } from "./types/agent";

import { IconHome, IconShield, IconSettings } from "./components/Icons";
import { StatusView } from "./components/StatusView";
import { UpdatesView } from "./components/UpdatesView";
import { SettingsView } from "./components/SettingsView";

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

const STATUS_POLL_MS = 5_000;
const STORAGE_KEY_VIEW = "wazuh_active_view";

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [agentStatus, setAgentStatus] = useState<AgentStatus>(DEFAULT_STATUS);
  const [metrics, setMetrics] = useState<SystemMetrics>(DEFAULT_METRICS);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [activeView, setActiveView] = useState<View>(() => {
    return (localStorage.getItem(STORAGE_KEY_VIEW) as View) || "status";
  });

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY_VIEW, activeView);
  }, [activeView]);

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig).catch(console.error);
    invoke<UpdateInfo>("check_for_updates").then(setUpdateInfo).catch(console.error);
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

  const indicatorTop = activeView === "status" 
    ? "34px" 
    : activeView === "updates" 
      ? "94px" 
      : "154px";

  return (
    <div className="app-wrapper" style={cssVars}>
      <nav className="sidebar">
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
  );
}

export default App;
