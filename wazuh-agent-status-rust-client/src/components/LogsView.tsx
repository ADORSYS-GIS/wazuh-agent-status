import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { LogLine } from "../types/agent";

export function LogsView() {
  const [logs, setLogs] = useState<LogLine[]>([]);
  const [filter, setFilter] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const logEndRef = useRef<HTMLDivElement>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  const startStream = useCallback(async () => {
    if (isStreaming) return;
    setIsStreaming(true);
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
      setLogs((prev) => [...prev, { raw: `[ERROR] Failed to start log stream: ${e}`, level: "ERROR" }]);
      setIsStreaming(false);
    });
  }, [isStreaming]);

  const stopStream = useCallback(() => {
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
    setIsStreaming(false);
  }, []);

  useEffect(() => {
    startStream();
    return () => {
      stopStream();
    };
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  const filteredLogs = logs.filter((log) => {
    if (!filter.trim()) return true;
    const term = filter.toLowerCase();
    return (
      log.raw.toLowerCase().includes(term) ||
      log.level.toLowerCase().includes(term)
    );
  });

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

  return (
    <div className="view-container">
      <div className="subtitle">Diagnostics</div>
      <h2 className="header title">Agent Logs</h2>

      <div style={{ display: "flex", gap: "8px", marginBottom: "16px", alignItems: "center" }}>
        <input
          type="text"
          placeholder="Filter logs (e.g. ERROR, WARNING)..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          style={{
            flex: 1,
            padding: "8px 12px",
            borderRadius: "6px",
            border: "1px solid var(--border)",
            background: "var(--bg-secondary)",
            color: "var(--text)",
            fontSize: "13px",
          }}
        />
        <button
          className="update-button"
          onClick={isStreaming ? stopStream : startStream}
          style={{ padding: "8px 16px", fontSize: "13px" }}
        >
          {isStreaming ? "Stop" : "Stream"}
        </button>
      </div>

      <div
        className="log-container"
        style={{
          background: "#0a0a0a",
          padding: "12px",
          borderRadius: "8px",
          fontSize: "11px",
          fontFamily: "monospace",
          maxHeight: "320px",
          overflowY: "auto",
          border: "1px solid #222",
          lineHeight: 1.5,
        }}
      >
        {filteredLogs.length === 0 ? (
          <div style={{ color: "#6b7280", textAlign: "center", padding: "20px" }}>
            {isStreaming ? "Waiting for log lines..." : "Stream stopped."}
          </div>
        ) : (
          filteredLogs.map((log, i) => (
            <div key={i} style={{ display: "flex", gap: "8px" }}>
              <span
                style={{
                  color: levelColor(log.level),
                  fontWeight: 700,
                  minWidth: "70px",
                  textTransform: "uppercase",
                  fontSize: "10px",
                  letterSpacing: "0.05em",
                  userSelect: "none",
                }}
              >
                {log.level}
              </span>
              <span style={{ color: "#e5e7eb", whiteSpace: "pre-wrap", wordBreak: "break-word" }}>
                {log.raw}
              </span>
            </div>
          ))
        )}
        <div ref={logEndRef} />
      </div>

      <div style={{ marginTop: "12px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span style={{ fontSize: "11px", color: "#9ca3af" }}>
          Showing {filteredLogs.length} of {logs.length} lines
        </span>
        <button
          className="update-button"
          onClick={() => setLogs([])}
          style={{ padding: "6px 12px", fontSize: "12px" }}
        >
          Clear
        </button>
      </div>
    </div>
  );
}
