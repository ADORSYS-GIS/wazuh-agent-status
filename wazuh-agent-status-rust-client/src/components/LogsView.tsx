import { useState, useEffect, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { LogLine } from "../types/agent";

interface LogsViewProps {
  readonly logs: LogLine[];
  readonly isStreaming: boolean;
  readonly error: string | null;
  readonly onStart: () => void;
  readonly onStop: () => void;
  readonly onClear: () => void;
}

export function LogsView({ logs, isStreaming, error, onStart, onStop, onClear }: LogsViewProps) {
  const [filter, setFilter] = useState("");
  const [downloadState, setDownloadState] = useState<"idle" | "loading" | "success" | "error">("idle");
  const [downloadMsg, setDownloadMsg] = useState<string | null>(null);
  const logContainerRef = useRef<HTMLDivElement>(null);

  const filteredLogs = useMemo(() => {
    return logs.filter((log) => {
      if (!filter.trim()) return true;
      const term = filter.toLowerCase();
      return (
        log.raw.toLowerCase().includes(term) ||
        log.level.toLowerCase().includes(term)
      );
    });
  }, [logs, filter]);

  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logs, filteredLogs.length]);

  const handleDownload = async () => {
    setDownloadState("loading");
    setDownloadMsg(null);
    try {
      // Take the last 100 lines of the in-memory streamed logs.
      // This avoids any privileged disk reads — the data was already
      // delivered through the server stream.
      const last100 = logs.slice(-100);
      const content = last100
        .map((l) => `[${l.level}] ${l.raw}`)
        .join("\n");

      const timestamp = Math.floor(Date.now() / 1000);
      const filename = `wazuh-ossec-logs-${timestamp}.txt`;

      const path = await invoke<string>("download_logs", { content, filename });
      setDownloadState("success");
      setDownloadMsg(`Saved to: ${path}`);
    } catch (e) {
      setDownloadState("error");
      setDownloadMsg(String(e));
    } finally {
      setTimeout(() => {
        setDownloadState("idle");
        setDownloadMsg(null);
      }, 4000);
    }
  };

  const levelColor = (level: string) => {
    switch (level) {
      case "ERROR":   return "#f87171";
      case "WARNING": return "#fbbf24";
      case "INFO":    return "#4ade80";
      case "DEBUG":   return "#60a5fa";
      default:        return "#d1d5db";
    }
  };

  let emptyMessage = null;
  if (isStreaming) {
    emptyMessage = "Waiting for log lines...";
  } else if (!error) {
    emptyMessage = "Click Stream to start.";
  }

  return (
    <div className="view-container">
      <div className="subtitle">Diagnostics</div>
      <h2 className="header title">Agent Logs</h2>

      <div className="logs-filter-row">
        <input
          className="logs-filter-input"
          type="text"
          placeholder="Filter logs (e.g. ERROR, WARNING)..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
        />
        <button
          type="button"
          className={`logs-stream-btn ${isStreaming ? "streaming" : ""}`}
          onClick={isStreaming ? onStop : onStart}
        >
          {isStreaming ? (
            <>
              <span className="logs-stream-pulse-dot" />{' '}
              Stop Streaming
            </>
          ) : (
            "Stream Logs"
          )}
        </button>
      </div>

      <div className="logs-container" ref={logContainerRef}>
        {error && (
          <div className="logs-error-banner">{error}</div>
        )}
        {filteredLogs.length === 0 ? (
          <div className="logs-empty">{emptyMessage}</div>
        ) : (
          filteredLogs.map((log, i) => (
            <div className="logs-line" key={`${log.level}-${log.raw}-${i}`}>
              <span className="logs-level" style={{ color: levelColor(log.level) }}>
                {log.level}
              </span>
              <span className="logs-message">{log.raw}</span>
            </div>
          ))
        )}
      </div>

      {downloadMsg && (
        <div className={`logs-download-feedback ${downloadState}`}>
          {downloadState === "success" ? "✓ " : "✕ "}
          {downloadMsg}
        </div>
      )}

      <div className="logs-footer">
        <span className="logs-count">
          Showing {filteredLogs.length} of {logs.length} lines
        </span>
        <div className="logs-footer-actions">
          {/* Download button is only shown while streaming */}
          {isStreaming && (() => {
            let iconAndText = (
              <>
                <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                  <polyline points="7 10 12 15 17 10" />
                  <line x1="12" y1="15" x2="12" y2="3" />
                </svg>
                {" "}Save Logs
              </>
            );
            if (downloadState === "loading") {
              iconAndText = (
                <>
                  <span className="logs-download-spinner" />
                  {" "}Saving…
                </>
              );
            } else if (downloadState === "success") {
              iconAndText = (
                <>
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                  {" "}Saved
                </>
              );
            }

            return (
              <button
                type="button"
                id="logs-download-btn"
                className={`logs-download-btn ${downloadState}`}
                onClick={handleDownload}
                disabled={downloadState === "loading"}
                title="Save last 100 log lines to your Downloads folder"
              >
                {iconAndText}
              </button>
            );
          })()}
          <button type="button" className="logs-clear-btn" onClick={onClear}>
            Clear
          </button>
        </div>
      </div>
    </div>
  );
}
