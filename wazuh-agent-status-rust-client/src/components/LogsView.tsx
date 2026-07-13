import { useState, useEffect, useRef } from "react";
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
  const logContainerRef = useRef<HTMLDivElement>(null);

  const filteredLogs = logs.filter((log) => {
    if (!filter.trim()) return true;
    const term = filter.toLowerCase();
    return (
      log.raw.toLowerCase().includes(term) ||
      log.level.toLowerCase().includes(term)
    );
  });

  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logs, filteredLogs.length]);

  const levelColor = (level: string) => {
    switch (level) {
      case "ERROR":
        return "#f87171";
      case "WARNING":
        return "#fbbf24";
      case "INFO":
        return "#4ade80";
      case "DEBUG":
        return "#60a5fa";
      default:
        return "#d1d5db";
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
          className={`logs-stream-btn ${isStreaming ? "streaming" : ""}`}
          onClick={isStreaming ? onStop : onStart}
        >
          {isStreaming ? (
            <>
              <span className="logs-stream-pulse-dot" />
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
              <span
                className="logs-level"
                style={{ color: levelColor(log.level) }}
              >
                {log.level}
              </span>
              <span className="logs-message">{log.raw}</span>
            </div>
          ))
        )}
      </div>

      <div className="logs-footer">
        <span className="logs-count">
          Showing {filteredLogs.length} of {logs.length} lines
        </span>
        <button className="logs-clear-btn" onClick={onClear}>
          Clear
        </button>
      </div>
    </div>
  );
}
