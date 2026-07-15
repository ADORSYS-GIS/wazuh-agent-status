import { useState, useCallback, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AiFixResult, ChatMessage } from "../types/ai";
import {
  parseMarkdownIntoChunks,
  commandNeedsSudo,
  commandIsInteractive,
  commandIsDestructive,
} from "../utils/compliance";

/** Shared terminal-output block used by command cards and SCA rescan. */
function TerminalOutput({ label, status, output }: Readonly<{ label: string; status: string; output: string }>) {
  return (
    <div className={`compliance-command-terminal ${status}`}>
      <div className="compliance-command-terminal-header">
        <span>{label}</span>
        <span className={`terminal-status-dot ${status}`} />
      </div>
      <pre className="compliance-command-terminal-log">{output}</pre>
    </div>
  );
}

interface ComplianceFixModalProps {
  fixResult: AiFixResult;
  onClose: () => void;
}

export function ComplianceFixModal({ fixResult, onClose }: Readonly<ComplianceFixModalProps>) {
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);
  const [commandStates, setCommandStates] = useState<Record<number, { status: "idle" | "running" | "success" | "failed"; output: string }>>({});
  const [scaRescanState, setScaRescanState] = useState<{ status: "idle" | "running" | "success" | "failed"; output: string }>({ status: "idle", output: "" });
  const messageIdRef = useRef(0);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [chatInput, setChatInput] = useState("");
  const [chatSending, setChatSending] = useState(false);

  const handleCopy = useCallback((content: string, index: number) => {
    navigator.clipboard.writeText(content);
    setCopiedIndex(index);
    setTimeout(() => {
      setCopiedIndex((prev) => (prev === index ? null : prev));
    }, 1500);
  }, []);

  const parsedData = useMemo(() => {
    return parseMarkdownIntoChunks(fixResult.markdown || "");
  }, [fixResult.markdown]);

  const handleRunCommand = useCallback(async (cmd: string, idx: number) => {
    setCommandStates((prev) => ({
      ...prev,
      [idx]: { status: "running", output: "Starting execution...\n" },
    }));

    try {
      let output: string;
      if (commandNeedsSudo(cmd)) {
        output = await invoke<string>("execute_fix_command_sudo", { command: cmd });
      } else {
        output = await invoke<string>("execute_fix_command", { command: cmd });
      }
      setCommandStates((prev) => ({
        ...prev,
        [idx]: { status: "success", output },
      }));
    } catch (e) {
      setCommandStates((prev) => ({
        ...prev,
        [idx]: { status: "failed", output: String(e) },
      }));
    }
  }, []);

  const handleSCARescan = useCallback(async () => {
    setScaRescanState({ status: "running", output: "Restarting wazuh-agent..." });
    try {
      const output = await invoke<string>("trigger_sca_rescan");
      setScaRescanState({ status: "success", output });
      onClose();
    } catch (e) {
      setScaRescanState({ status: "failed", output: String(e) });
    }
  }, [onClose]);

  const handleChatSend = useCallback(async () => {
    const msg = chatInput.trim();
    if (!msg || chatSending) return;

    const msgId = ++messageIdRef.current;
    setChatInput("");
    setChatMessages((prev) => [...prev, { id: msgId, role: "user", content: msg }]);
    setChatSending(true);

    try {
      const reply = await invoke<string>("ai_chat", {
        prompt: msg,
        context: fixResult.markdown || null,
      });
      const replyId = ++messageIdRef.current;
      setChatMessages((prev) => [...prev, { id: replyId, role: "assistant", content: reply }]);
    } catch (e) {
      const errId = ++messageIdRef.current;
      setChatMessages((prev) => [
        ...prev,
        { id: errId, role: "assistant", content: `Error: ${e}` },
      ]);
    } finally {
      setChatSending(false);
    }
  }, [chatInput, chatSending, fixResult.markdown]);

  return (
    <div className="update-modal-backdrop">
      <button
        type="button"
        className="update-modal-backdrop-close-button"
        onClick={onClose}
        aria-label="Close modal"
        style={{
          position: "absolute",
          inset: 0,
          background: "transparent",
          border: "none",
          cursor: "default",
        }}
      />
      <div className="ai-fix-modal" style={{ position: "relative", zIndex: 1 }}>
        <div className="update-modal-header">
          <div className="update-modal-title">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ color: "var(--primary)" }}>
              <path d="M12 2l2.4 7.2H22l-6 4.8 2.4 7.2L12 16l-6 4.8L8.4 14l-6-4.8h7.6z" />
            </svg>
            {fixResult.success ? "AI Fix Suggestions" : "Fix Generation Failed"}
          </div>
          <button onClick={onClose} className="compliance-refresh-btn">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        {fixResult.success ? (
          <div className="ai-fix-modal-panes">
            {/* Left Pane */}
            <div className="ai-fix-modal-left">
              <div className="ai-fix-modal-left-scroll">
                {/* Risk & Impact Alert */}
                {(parsedData.riskLevel || parsedData.impact) && (
                  <div className={`compliance-fix-impact-card ${parsedData.riskLevel?.toLowerCase() || "low"}`}>
                    <div className="compliance-fix-impact-header">
                      <div className="compliance-fix-risk-title">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
                          <line x1="12" y1="9" x2="12" y2="13" />
                          <line x1="12" y1="17" x2="12.01" y2="17" />
                        </svg>
                        Risk Evaluation
                      </div>
                      <span className={`compliance-fix-risk-badge ${parsedData.riskLevel?.toLowerCase() || "low"}`}>
                        {parsedData.riskLevel || "Low"}
                      </span>
                    </div>
                    {parsedData.impact && (
                      <div className="compliance-fix-impact-desc">{parsedData.impact}</div>
                    )}
                  </div>
                )}

                <div className="ai-fix-markdown">
                  {parsedData.chunks.map((chunk, i) => {
                    const key = `${chunk.type}-${chunk.content.slice(0, 40)}`;
                    if (chunk.type === "heading2") {
                      return <h3 key={key} className="ai-fix-heading">{chunk.content}</h3>;
                    }
                    if (chunk.type === "heading3") {
                      return <h4 key={key} className="ai-fix-subheading">{chunk.content}</h4>;
                    }
                    if (chunk.type === "step") {
                      return <div key={key} className="ai-fix-step">{chunk.content}</div>;
                    }
                    if (chunk.type === "list_item") {
                      return <li key={key} className="ai-fix-list-item">{chunk.content}</li>;
                    }
                    if (chunk.type === "text") {
                      return <p key={key} className="ai-fix-paragraph">{chunk.content}</p>;
                    }
                    if (chunk.type === "code_block") {
                      const execState = commandStates[i] || { status: "idle", output: "" };
                      return (
                        <div key={key} className={`compliance-command-card ${execState.status}`}>
                          <div className="compliance-command-card-header">
                            <span className="compliance-command-title">Suggested Shell Command</span>
                            <div className="compliance-command-actions">
                              <button
                                className={`ai-copy-btn ${copiedIndex === i ? "copied" : ""}`}
                                onClick={() => handleCopy(chunk.content, i)}
                                title="Copy Command"
                              >
                                {copiedIndex === i ? (
                                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                    <polyline points="20 6 9 17 4 12" />
                                  </svg>
                                ) : (
                                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                                  </svg>
                                )}
                              </button>
                              <button
                                className="compliance-command-run-btn"
                                onClick={() => handleRunCommand(chunk.content, i)}
                                disabled={execState.status === "running" || commandIsInteractive(chunk.content)}
                              >
                                {execState.status === "running" ? (
                                  <span className="settings-ai-spinner" />
                                ) : (
                                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                                    <polygon points="5 3 19 12 5 21 5 3" />
                                  </svg>
                                )}
                                {execState.status === "running" ? "Running..." : "Execute"}
                              </button>
                            </div>
                          </div>

                          {commandIsInteractive(chunk.content) && (
                            <div className="compliance-command-interactive-warn">
                              ⚠ This command opens an interactive editor and cannot run inside the app. Copy it and run it in a terminal.
                            </div>
                          )}

                          {commandIsDestructive(chunk.content) && !commandIsInteractive(chunk.content) && (
                            <div className="compliance-command-interactive-warn" style={{ borderColor: "var(--error)", color: "var(--error)" }}>
                              ⚠ This command may be destructive (e.g. deletes files, writes to a disk, or wipes data). Review carefully before executing.
                            </div>
                          )}

                          <pre className="compliance-command-text">
                            <code>{chunk.content}</code>
                          </pre>

                          {commandNeedsSudo(chunk.content) && !commandIsInteractive(chunk.content) && (
                            <div className="compliance-sudo-row" style={{ fontStyle: "italic", fontSize: "0.7rem", color: "var(--text-dim)", opacity: 0.8 }}>
                              ℹ Requires elevation. System will prompt to approve.
                            </div>
                          )}

                          {execState.output && (
                            <TerminalOutput label="Console Output" status={execState.status} output={execState.output} />
                          )}
                        </div>
                      );
                    }
                    return null;
                  })}
                </div>
              </div>

              {/* SCA Rescan - pinned at bottom */}
              <div className="sca-rescan-card" style={{ flexShrink: 0 }}>
                <div className="sca-rescan-card-header">
                  <div className="sca-rescan-card-title">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                      <polyline points="23 4 23 10 17 10" />
                      <polyline points="1 20 1 14 7 14" />
                      <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
                    </svg>
                    Trigger SCA Rescan
                  </div>
                  <span className="sca-rescan-card-hint">Restarts wazuh-agent to force a fresh compliance scan</span>
                </div>

                <div className="sca-rescan-password-row">
                  <span style={{ fontSize: "0.7rem", color: "var(--text-dim)", fontStyle: "italic" }}>
                    ℹ This action will prompt you for administrator privileges.
                  </span>
                  <button
                    className="compliance-command-run-btn"
                    onClick={handleSCARescan}
                    disabled={scaRescanState.status === "running"}
                  >
                    {scaRescanState.status === "running" ? (
                      <><span className="settings-ai-spinner" /> Restarting...</>
                    ) : (
                      <>
                        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                          <polygon points="5 3 19 12 5 21 5 3" />
                        </svg>
                        Rescan Now
                      </>
                    )}
                  </button>
                </div>

                {scaRescanState.output && (
                  <TerminalOutput label="Restart Output" status={scaRescanState.status} output={scaRescanState.output} />
                )}
              </div>
            </div>

            {/* Right Pane: Follow-up Chat */}
            <div className="ai-fix-modal-right">
              <div className="ai-chat-section">
                <div className="ai-chat-divider">
                  <span>Ask a follow-up question</span>
                </div>

                <div className="ai-chat-messages">
                  {chatMessages.length === 0 && (
                    <div className="ai-chat-empty-state">
                      Ask any questions or request clarification about these steps here.
                    </div>
                  )}
                  {chatMessages.map((m) => (
                    <div key={m.id} className={`ai-chat-msg ${m.role}`}>
                      <div className="ai-chat-msg-role">{m.role === "user" ? "You" : "AI"}</div>
                      <div className="ai-chat-msg-content">{m.content}</div>
                    </div>
                  ))}
                  {chatSending && (
                    <div className="ai-chat-typing-indicator">
                      <div className="ai-chat-typing-dot" />
                      <div className="ai-chat-typing-dot" />
                      <div className="ai-chat-typing-dot" />
                    </div>
                  )}
                </div>

                <div className="ai-chat-input-row">
                  <textarea
                    className="ai-chat-input"
                    placeholder="Ask a follow-up question..."
                    rows={2}
                    value={chatInput}
                    onChange={(e) => setChatInput(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        handleChatSend();
                      }
                    }}
                    disabled={chatSending}
                    spellCheck={false}
                  />
                  <button
                    className="ai-chat-send-btn"
                    onClick={handleChatSend}
                    disabled={!chatInput.trim() || chatSending}
                  >
                    {chatSending ? (
                      <span className="settings-ai-spinner" />
                    ) : (
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                        <line x1="22" y1="2" x2="11" y2="13" />
                        <polygon points="22 2 15 22 11 13 2 9 22 2" />
                      </svg>
                    )}
                  </button>
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="ai-fix-modal-body">
            <div className="compliance-error">
              <div className="compliance-error-icon">!</div>
              <div className="compliance-error-title">Failed to generate fix</div>
              <div className="compliance-error-text">{fixResult.error}</div>
            </div>
          </div>
        )}

        <button className="update-modal-dismiss" onClick={onClose}>
          Close
        </button>
      </div>
    </div>
  );
}
