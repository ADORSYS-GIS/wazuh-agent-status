import { useState, useEffect, useRef, JSX } from "react";
import { invoke } from "@tauri-apps/api/core";

interface LogEntry {
  id: string;
  text: string;
}

interface UpdateModalProps {
  status: "running" | "success" | "error";
  logs: LogEntry[];
  targetVersion: string;
  onDismiss: () => void;
}

type SaveState = "idle" | "loading" | "success" | "error";

type Step = "connecting" | "preparing" | "downloading" | "installing" | "done" | "failed";

function inferStep(logs: LogEntry[]): Step {
  if (logs.length === 0) return "connecting";

  // Terminal states are driven by the last line only.
  // Only the server's final "[SUCCESS] Update completed successfully" message counts,
  // not intermediate [SUCCESS] lines from the script (e.g. "Installation validated successfully").
  const last = logs[logs.length - 1]?.text ?? "";
  if (last.includes("UPDATE_PROGRESS: [SUCCESS] Update completed successfully")) return "done";
  // Only [FAILURE] triggers failed — [ERROR] lines are intermediate (e.g. stderr from the script)
  if (last.includes("[FAILURE]")) return "failed";

  // Monotonic progression: scan ALL logs to find the highest step reached,
  // so progress only moves forward and never jumps back (fixes UI blinking)
  const order: Step[] = ["connecting", "preparing", "downloading", "installing"];
  let highest = 0;

  for (const log of logs) {
    const text = log.text ?? "";
    let step: Step = "preparing";
    if (text.toLowerCase().includes("download")) step = "downloading";
    else if (text.toLowerCase().includes("install") || text.toLowerCase().includes("setup") || text.toLowerCase().includes("execut")) step = "installing";

    const idx = order.indexOf(step);
    if (idx > highest) highest = idx;
  }

  return order[highest];
}

const STEP_LABELS: Record<Step, string> = {
  connecting: "Connecting to server...",
  preparing: "Preparing update...",
  downloading: "Downloading package...",
  installing: "Installing...",
  done: "Update complete!",
  failed: "Update failed",
};

function StepIndicator({ step, current }: Readonly<{ step: Step; current: Step }>) {
  const order: Step[] = ["connecting", "preparing", "downloading", "installing", "done"];
  const idx = order.indexOf(step);
  const curIdx = order.indexOf(current);
  const isActive = step === current;
  const isComplete = current === "done"
    ? true
    : (curIdx > idx && current !== "failed");
  const isFailed = current === "failed";

  let icon: JSX.Element;
if (isComplete) {
  icon = (
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
      <path d="M3 7.5L5.5 10L11 4" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
    </svg>
  );
} else if (isActive && current === "failed") {
  icon = (
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
      <path d="M4 4L10 10M10 4L4 10" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
    </svg>
  );
} else if (isActive) {
  icon = <div className="spinner-ring" />;
} else {
  icon = <div className="step-dot" />;
}


  return (
    <div className={`update-step ${isActive ? "active" : ""} ${isComplete ? "complete" : ""} ${isFailed && isActive ? "failed" : ""} ${isFailed && !isActive && !isComplete ? "failed" : ""}`}>
      <div className="step-icon">
        {icon}
      </div>
      <span className="step-label">{STEP_LABELS[step]}</span>
    </div>
  );
}

export function UpdateModal({ status, logs, targetVersion, onDismiss }: Readonly<UpdateModalProps>) {
  const [showTerminal, setShowTerminal] = useState(false);
  const [startedAt] = useState(Date.now());
  const [elapsed, setElapsed] = useState(0);
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const [saveMsg, setSaveMsg] = useState<string | null>(null);
  const logEndRef = useRef<HTMLDivElement>(null);
  const currentStep = inferStep(logs);

  const handleSaveLogs = async () => {
    setSaveState("loading");
    setSaveMsg(null);
    try {
      const content = logs.map((l) => l.text).join("\n");
      const timestamp = Math.floor(Date.now() / 1000);
      const filename = `wazuh-update-logs-${timestamp}.txt`;
      const path = await invoke<string>("download_logs", { content, filename });
      setSaveState("success");
      setSaveMsg(`Saved to: ${path}`);
    } catch (e) {
      setSaveState("error");
      setSaveMsg(String(e));
    } finally {
      setTimeout(() => { setSaveState("idle"); setSaveMsg(null); }, 4000);
    }
  };

  useEffect(() => {
    if (status === "running") {
      const timer = setInterval(() => setElapsed(Date.now() - startedAt), 1000);
      return () => clearInterval(timer);
    }
  }, [status, startedAt]);

  useEffect(() => {
    if (showTerminal) {
      logEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs, showTerminal]);

  const formatElapsed = (ms: number) => {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return m > 0 ? `${m}m ${sec}s` : `${sec}s`;
  };

  const steps: Step[] = ["connecting", "preparing", "downloading", "installing", "done"];

  return (
    <div className="update-modal-backdrop">
      <div className="update-modal">
        {/* Fixed header */}
        <div className="update-modal-header">
          <div className="update-modal-title">
            <span className={`update-status-badge ${status}`}>{status.toUpperCase()}</span>
            <span>Updating to v{targetVersion}</span>
          </div>
          {status === "running" && (
            <span className="update-elapsed">{formatElapsed(elapsed)}</span>
          )}
        </div>

        {/* Scrollable body */}
        <div className="update-modal-body">
          {/* Progress steps */}
          <div className="update-steps">
            {steps.map((step) => (
              <StepIndicator key={step} step={step} current={currentStep} />
            ))}
          </div>

          {/* Simplified status message */}
          <div className="update-current-action">
            {currentStep === "done" && (
              <div className="update-result success-result">
                <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                  <circle cx="10" cy="10" r="9" stroke="currentColor" strokeWidth="2"/>
                  <path d="M6 10.5L8.5 13L14 7" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
                <span>Update completed successfully</span>
              </div>
            )}
            {currentStep === "failed" && (
              <div className="update-result error-result">
                <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                  <circle cx="10" cy="10" r="9" stroke="currentColor" strokeWidth="2"/>
                  <path d="M7 7L13 13M13 7L7 13" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                </svg>
                <span>Update failed — see details below</span>
              </div>
            )}
          </div>

          {/* Expandable terminal */}
          {logs.length > 0 && (
            <div className="update-terminal-wrapper">
              <button
                type="button"
                className="update-terminal-toggle"
                onClick={() => setShowTerminal(!showTerminal)}
              >
                <svg
                  width="12"
                  height="12"
                  viewBox="0 0 12 12"
                  fill="none"
                  style={{
                    transform: showTerminal ? "rotate(90deg)" : "rotate(0deg)",
                    transition: "transform 0.2s ease"
                  }}
                >
                  <path d="M4 3L7 6L4 9" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
                </svg>
                <span>Details ({logs.length} lines)</span>
              </button>
              {showTerminal && (
                <div className="update-terminal">
                  {logs.map((log) => {
                    const isError = log.text.includes("[ERROR]") || log.text.includes("[FAILURE]");
                    const isSuccess = log.text.includes("[SUCCESS]");
                    const isStatus = log.text.includes("[STATUS]");
                    return (
                      <div
                        key={log.id}
                        className={`terminal-line ${isError ? "error" : ""} ${isSuccess ? "success" : ""} ${isStatus ? "status" : ""}`}
                      >
                        {log.text.replace(/UPDATE_PROGRESS:\s*/g, "")}
                      </div>
                    );
                  })}
                  <div ref={logEndRef} />
                </div>
              )}
            </div>
          )}
        </div>

        {/* Fixed footer: Save Logs + Dismiss — shown when update is no longer running */}
        {status !== "running" && (
          <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
            {saveMsg && (
              <div className={`logs-download-feedback ${saveState}`} style={{ fontSize: "10px" }}>
                {saveMsg}
              </div>
            )}
            <div style={{ display: "flex", gap: "8px" }}>
              <button
                type="button"
                className={`logs-download-btn ${saveState}`}
                style={{ fontSize: "12px", padding: "8px 14px" }}
                onClick={handleSaveLogs}
                disabled={saveState === "loading"}
                title="Save update logs to your Downloads folder"
              >
                {saveState === "loading" ? (
                  <><span className="logs-download-spinner" /> Saving…</>
                ) : saveState === "success" ? (
                  <>
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                      <polyline points="20 6 9 17 4 12" />
                    </svg>
                    Saved
                  </>
                ) : (
                  <>
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                      <polyline points="7 10 12 15 17 10" />
                      <line x1="12" y1="15" x2="12" y2="3" />
                    </svg>
                    Save Logs
                  </>
                )}
              </button>
              <button type="button" className="update-modal-dismiss" style={{ flex: 1 }} onClick={onDismiss}>
                {currentStep === "done" ? "Done" : "Close"}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
