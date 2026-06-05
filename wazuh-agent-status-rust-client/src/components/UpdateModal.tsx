import { useState, useEffect, useRef } from "react";

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

function StepIndicator({ step, current }: { step: Step; current: Step }) {
  const order: Step[] = ["connecting", "preparing", "downloading", "installing", "done"];
  const idx = order.indexOf(step);
  const curIdx = order.indexOf(current);
  const isActive = step === current;
  const isComplete = current === "done"
    ? true
    : (curIdx > idx && current !== "failed");
  const isFailed = current === "failed";

  return (
    <div className={`update-step ${isActive ? "active" : ""} ${isComplete ? "complete" : ""} ${isFailed && isActive ? "failed" : ""} ${isFailed && !isActive && !isComplete ? "" : ""}`}>
      <div className="step-icon">
        {isComplete ? (
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M3 7.5L5.5 10L11 4" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        ) : isActive && current === "failed" ? (
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M4 4L10 10M10 4L4 10" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
          </svg>
        ) : isActive ? (
          <div className="spinner-ring" />
        ) : (
          <div className="step-dot" />
        )}
      </div>
      <span className="step-label">{STEP_LABELS[step]}</span>
    </div>
  );
}

export function UpdateModal({ status, logs, targetVersion, onDismiss }: Readonly<UpdateModalProps>) {
  const [showTerminal, setShowTerminal] = useState(false);
  const [startedAt] = useState(Date.now());
  const [elapsed, setElapsed] = useState(0);
  const logEndRef = useRef<HTMLDivElement>(null);
  const currentStep = inferStep(logs);

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

        {/* Fixed dismiss button */}
        {status !== "running" && (
          <button className="update-modal-dismiss" onClick={onDismiss}>
            {currentStep === "done" ? "Done" : "Close"}
          </button>
        )}
      </div>
    </div>
  );
}
