import { useState, useEffect, useRef, useCallback, useMemo, type CSSProperties } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

import type { AppConfig, View } from "./types/app";
import type { AgentStatus, SystemMetrics, UpdateStatus, LogLine } from "./types/agent";

import { IconHome, IconLogs, IconShield, IconSettings, IconCheckSquare } from "./components/Icons";
import { StatusView } from "./components/StatusView";
import { LogsView } from "./components/LogsView";
import { UpdatesView } from "./components/UpdatesView";
import { SettingsView } from "./components/SettingsView";
import { ComplianceView } from "./components/ComplianceView";
import { UpdateAvailableModal } from "./components/UpdateAvailableModal";

import { computeBrandCSS, getBrandLogoUrl } from "./brand";

// ─── Defaults ─────────────────────────────────────────────────────────────────

const DEFAULT_STATUS: AgentStatus = {
  status: "Unknown",
  connection: "Disconnected",
  version: "Unknown",
  tray_version: "Unknown",
  groups: [],
  self_healing_enabled: true,
  agent_id: "",
  agent_name: "",
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
const UPDATE_POLL_MS = 5 * 60 * 1000;
const STORAGE_KEY_VIEW = "wazuh_active_view";

const IS_WINDOWS = typeof navigator !== "undefined"
  && navigator.userAgent.toLowerCase().includes("windows");

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [agentStatus, setAgentStatus] = useState<AgentStatus>(DEFAULT_STATUS);
  const [metrics, setMetrics] = useState<SystemMetrics>(DEFAULT_METRICS);
  const [updateInfo, setUpdateInfo] = useState<UpdateStatus | null>(null);
  const [showUpdatePrompt, setShowUpdatePrompt] = useState(false);
  const [activeView, setActiveView] = useState<View>(() => {
    return (localStorage.getItem(STORAGE_KEY_VIEW) as View) || "status";
  });

  // Global log stream state (persists across navigation)
  const [logs, setLogs] = useState<LogLine[]>([]);
  const [isLogStreaming, setIsLogStreaming] = useState(false);
  const [logError, setLogError] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const mainContentRef = useRef<HTMLDivElement>(null);
  const lastNotifiedUpdateVersionRef = useRef<string | null>(null);

  const startLogStream = useCallback(async () => {
    if (isLogStreaming) return;
    setIsLogStreaming(true);
    setLogError(null);
    setLogs([]);

    const unlisten = await listen<string>("log-line", (event) => {
      try {
        const parsed: LogLine = JSON.parse(event.payload);
        setLogs((prev) => [...prev, parsed]);
      } catch {
        setLogs((prev) => [...prev, { raw: event.payload, level: "UNKNOWN" }]);
      }
    });

    unlistenRef.current = unlisten;
    invoke("start_log_stream").catch((e) => {
      setLogError(`Failed to start log stream: ${e}`);
      setIsLogStreaming(false);
    });
  }, [isLogStreaming]);

  const stopLogStream = useCallback(() => {
    if (unlistenRef.current) { unlistenRef.current(); unlistenRef.current = null; }
    setIsLogStreaming(false);
  }, []);

  const refreshUpdateInfo = useCallback(() => {
    invoke<UpdateStatus>("check_for_updates").then(setUpdateInfo).catch(console.error);
  }, []);

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY_VIEW, activeView);
    if (mainContentRef.current) {
      mainContentRef.current.scrollTo(0, 0);
    }
  }, [activeView]);

  useEffect(() => {
    if (!IS_WINDOWS) {
      setShowUpdatePrompt(false);
      lastNotifiedUpdateVersionRef.current = null;
      return;
    }

    if (!updateInfo?.has_updates) {
      setShowUpdatePrompt(false);
      lastNotifiedUpdateVersionRef.current = null;
      return;
    }

    setShowUpdatePrompt(true);
  }, [activeView, updateInfo]);

  useEffect(() => {
    if (!IS_WINDOWS) {
      return;
    }

    if (!updateInfo?.has_updates) {
      lastNotifiedUpdateVersionRef.current = null;
      return;
    }

    if (lastNotifiedUpdateVersionRef.current === updateInfo.tray.latest_version) {
      return;
    }

    lastNotifiedUpdateVersionRef.current = updateInfo.tray.latest_version;
    setShowUpdatePrompt(true);

    invoke("notify_update_available", {
      currentVersion: updateInfo.tray.current_version,
      latestVersion: updateInfo.tray.latest_version,
    }).catch(console.error);
  }, [activeView, updateInfo]);

  const openUpdatesFromPrompt = useCallback(() => {
    setShowUpdatePrompt(false);
    setActiveView("updates");
  }, [updateInfo]);

  const remindLaterFromPrompt = useCallback(() => {
    setShowUpdatePrompt(false);
  }, [updateInfo]);

  // Keep a ref of the last known tray_version so we can detect changes
  const prevTrayVersionRef = useRef<string | null>(null);

  useEffect(() => {
    // Initial data fetch
    invoke<AppConfig>("get_config")
      .then(setConfig)
      .catch(console.error);
    refreshUpdateInfo();

    // Polling logic for real-time data
    const refreshData = () => {
      invoke<AgentStatus>("get_agent_status").then(setAgentStatus).catch(console.error);
      invoke<SystemMetrics>("get_system_metrics").then(setMetrics).catch(console.error);
    };

    refreshData();
    const statusTimer = setInterval(refreshData, STATUS_POLL_MS);
    const updateTimer = setInterval(refreshUpdateInfo, UPDATE_POLL_MS);

    return () => {
      clearInterval(statusTimer);
      clearInterval(updateTimer);
      if (unlistenRef.current) { unlistenRef.current(); unlistenRef.current = null; }
    };
  }, [refreshUpdateInfo]);

  // Reactive: when agentStatus.tray_version changes, refresh the version check info
  // This ensures Health & Updates auto-updates like Settings without extra polling
  useEffect(() => {
    if (prevTrayVersionRef.current !== null && prevTrayVersionRef.current !== agentStatus.tray_version) {
      refreshUpdateInfo();
    }
    prevTrayVersionRef.current = agentStatus.tray_version;
  }, [agentStatus.tray_version, refreshUpdateInfo]);

  useEffect(() => {
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
    };
    document.addEventListener("contextmenu", handleContextMenu);
    return () => document.removeEventListener("contextmenu", handleContextMenu);
  }, []);

  // ── Brand-driven CSS variables (MUST be before early return — hooks rule) ──
  const cssVars = useMemo(() => {
    if (!config) return {};
    return computeBrandCSS(config.brand);
  }, [config]);

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

  const activeViewIndex = { status: 0, compliance: 1, logs: 2, updates: 3, settings: 4 }[activeView] ?? 0;

  return (
    <div className="app-wrapper" style={cssVars as CSSProperties}>
      {showUpdatePrompt && updateInfo && (
        <UpdateAvailableModal
          currentVersion={updateInfo.tray.current_version}
          latestVersion={updateInfo.tray.latest_version}
          onOpenUpdates={openUpdatesFromPrompt}
          onRemindLater={remindLaterFromPrompt}
        />
      )}

      <nav className="sidebar">
        <div className="sidebar-logo">
          <img src={getBrandLogoUrl(config.brand)} alt={config.brand.company} />
        </div>

        <div 
          className="nav-items" 
          style={{ 
            position: 'relative', 
            display: 'flex', 
            flexDirection: 'column', 
            alignItems: 'center',
            "--nav-active-index": activeViewIndex
          } as CSSProperties}
        >
          <div className="nav-indicator" />
          
          <div className="tooltip-container">
            <button
              type="button"
              className={`nav-item ${activeView === "status" ? "active" : ""}`}
              onClick={() => setActiveView("status")}
              aria-label="Overview"
            >
              <IconHome />
              <span className="nav-label">Overview</span>
            </button>
            <span className="tooltip">Overview</span>
          </div>

          <div className="tooltip-container">
            <button
              type="button"
              className={`nav-item ${activeView === "compliance" ? "active" : ""}`}
              onClick={() => setActiveView("compliance")}
              aria-label="Compliance"
            >
              <IconCheckSquare />
              <span className="nav-label">Compliance</span>
            </button>
            <span className="tooltip">Compliance</span>
          </div>

          <div className="tooltip-container">
            <button
              type="button"
              className={`nav-item ${activeView === "logs" ? "active" : ""}`}
              onClick={() => setActiveView("logs")}
              aria-label="Logs"
            >
              <IconLogs />
              <span className="nav-label">Logs</span>
            </button>
            <span className="tooltip">Logs</span>
          </div>

          <div className="tooltip-container">
            <button
              type="button"
              className={`nav-item ${activeView === "updates" ? "active" : ""}`}
              onClick={() => setActiveView("updates")}
              aria-label="Health & Updates"
            >
              <IconShield />
              <span className="nav-label">Health & Updates</span>
              {IS_WINDOWS && updateInfo?.has_updates && <span className="notification-dot" />}
            </button>
            <span className="tooltip">Health & Updates</span>
          </div>

          <div className="tooltip-container">
            <button
              type="button"
              className={`nav-item ${activeView === "settings" ? "active" : ""}`}
              onClick={() => setActiveView("settings")}
              aria-label="Settings"
            >
              <IconSettings />
              <span className="nav-label">Settings</span>
            </button>
            <span className="tooltip">Settings</span>
          </div>
        </div>

        <div className="sidebar-bottom">
          <div className="copyright">© {new Date().getFullYear()} {config.brand.company}</div>
        </div>
      </nav>

      <main className="main-content" ref={mainContentRef}>
        {activeView === "status" && <StatusView agentStatus={agentStatus} metrics={metrics} config={config} />}
        {activeView === "logs" && (
          <LogsView
            logs={logs}
            isStreaming={isLogStreaming}
            error={logError}
            onStart={startLogStream}
            onStop={stopLogStream}
            onClear={() => setLogs([])}
          />
        )}
        {activeView === "updates" && (
          <UpdatesView 
            updateInfo={updateInfo} 
            agentStatus={agentStatus} 
            onRefreshUpdates={refreshUpdateInfo} 
          />
        )}
        {activeView === "compliance" && <ComplianceView agentStatus={agentStatus} />}
        {activeView === "settings" && <SettingsView config={config} />}
      </main>
    </div>
  );
}

export default App;
