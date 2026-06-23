import { useState, useEffect, useCallback, useMemo, type CSSProperties } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AgentStatus, ComplianceReport, ComplianceCheckResult } from "../types/agent";
import type { AiFixResult, AiProviderStatus, FailedCheckInput, ChatMessage } from "../types/ai";

// ─── Markdown Parser ──────────────────────────────────────────────────────────

interface MarkdownChunk {
  type: "text" | "heading2" | "heading3" | "step" | "list_item" | "code_block";
  content: string;
  language?: string;
}

function parseMarkdownIntoChunks(markdown: string) {
  const chunks: MarkdownChunk[] = [];
  let riskLevel: string | null = null;
  let impact: string | null = null;

  if (!markdown) return { chunks, riskLevel, impact };

  const parts = markdown.split(/(```[\s\S]*?```)/g);

  for (const part of parts) {
    if (part.startsWith("```")) {
      const match = part.match(/```(bash|sh|powershell|cmd|shell|zsh)?\n([\s\S]*?)```/);
      if (match) {
        const language = match[1] || "bash";
        const content = match[2].trim();
        chunks.push({ type: "code_block", content, language });
      } else {
        const content = part.replace(/```/g, "").trim();
        chunks.push({ type: "code_block", content, language: "bash" });
      }
    } else {
      const lines = part.split("\n");
      let currentSection: "risk" | "impact" | null = null;

      for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed === "") continue;

        if (trimmed.startsWith("## Risk Level")) {
          currentSection = "risk";
          continue;
        } else if (trimmed.startsWith("## Impact")) {
          currentSection = "impact";
          continue;
        } else if (trimmed.startsWith("## ")) {
          currentSection = null;
          chunks.push({ type: "heading2", content: trimmed.replace("## ", "") });
        } else if (trimmed.startsWith("### ")) {
          currentSection = null;
          chunks.push({ type: "heading3", content: trimmed.replace("### ", "") });
        } else if (/^\d+\.\s/.test(trimmed)) {
          currentSection = null;
          chunks.push({ type: "step", content: trimmed });
        } else if (trimmed.startsWith("- ") || trimmed.startsWith("* ")) {
          currentSection = null;
          chunks.push({ type: "list_item", content: trimmed.replace(/^[-*]\s+/, "") });
        } else {
          if (currentSection === "risk") {
            riskLevel = trimmed;
          } else if (currentSection === "impact") {
            impact = (impact ? impact + " " : "") + trimmed;
          } else {
            chunks.push({ type: "text", content: trimmed });
          }
        }
      }
    }
  }

  return { chunks, riskLevel, impact };
}

// ─── Constants ────────────────────────────────────────────────────────────────

function scoreColor(score: number): string {
  if (score >= 80) return "var(--success)";
  if (score >= 50) return "var(--warning)";
  return "var(--error)";
}

function scoreLabel(score: number): string {
  if (score >= 80) return "Good";
  if (score >= 50) return "Fair";
  return "Poor";
}

// ─── Main Component ───────────────────────────────────────────────────────────

export function ComplianceView({ agentStatus }: { agentStatus: AgentStatus }) {
  const [report, setReport] = useState<ComplianceReport | null>(null);
  const [agentInfo, setAgentInfo] = useState<{ id: string; name: string } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedCategory, setExpandedCategory] = useState<string | null>(null);
  const [animateScore, setAnimateScore] = useState(false);

  // Client-side filter state (no server re-fetch)
  const [localFilter, setLocalFilter] = useState<string>("all");

  // Last-updated timestamp
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  // ── AI Fix State ──────────────────────────────────────────────────────────
  const [aiStatus, setAiStatus] = useState<AiProviderStatus | null>(null);
  const [fixingCheck, setFixingCheck] = useState<string | null>(null); // check title being fixed
  const [fixResult, setFixResult] = useState<AiFixResult | null>(null);
  const [fixingAll, setFixingAll] = useState(false);
  // ── Follow-up Chat State ──────────────────────────────────────────────────
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [chatInput, setChatInput] = useState("");
  const [chatSending, setChatSending] = useState(false);
  // ── Copy Feedback State ───────────────────────────────────────────────────
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);
  // ── Command Execution State ────────────────────────────────────────────────
  const [commandStates, setCommandStates] = useState<Record<number, { status: "idle" | "running" | "success" | "failed"; output: string }>>({});
  const [sharedPassword, setSharedPassword] = useState("");
  // ── SCA Rescan State ──────────────────────────────────────────────────────
  const [scaRescanState, setScaRescanState] = useState<{ status: "idle" | "running" | "success" | "failed"; output: string }>({ status: "idle", output: "" });
  const [showRefreshAfterRescan, setShowRefreshAfterRescan] = useState(false);

  const parsedData = useMemo(() => {
    return parseMarkdownIntoChunks(fixResult?.markdown || "");
  }, [fixResult?.markdown]);

  // Load AI provider status (never fails — returns { configured: false } if unset)
  useEffect(() => {
    invoke<AiProviderStatus>("get_ai_status").then(setAiStatus);
  }, []);

  // ── Fetch data ──────────────────────────────────────────────────────────

  const fetchReport = useCallback(async () => {
    try {
      setError(null);

      const agentId = agentStatus.agent_id;
      const agentName = agentStatus.agent_name;

      setAgentInfo({ id: agentId, name: agentName });

      const result = await invoke<ComplianceReport>("fetch_compliance", {
        agentId,
        statusFilter: null,
        mandatory: null,
        category: null,
      });

      setReport(result);
      setLastUpdated(new Date());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [agentStatus.agent_id, agentStatus.agent_name]);

  // Fetch only after the server has provided a valid agent ID (avoids fallback flash)
  useEffect(() => {
    if (agentStatus.agent_id) {
      fetchReport();
    }
  }, [fetchReport, agentStatus.agent_id]);

  // Trigger score ring animation AFTER data loads (not on mount — avoids animating while skeleton is shown)
  useEffect(() => {
    if (!loading && report) {
      requestAnimationFrame(() => setAnimateScore(true));
    }
  }, [loading, report]);

  // ── Client-side filtering ───────────────────────────────────────────────

  const filteredReport = useMemo(() => {
    if (!report) return null;

    if (localFilter === "all") return report;

    const filteredCategories = report.categories
      .map((cat) => ({
        ...cat,
        checks: cat.checks.filter((chk) => {
          if (localFilter === "passed") return chk.status === "Passed";
          if (localFilter === "failed") return chk.status === "Failed";
          return true;
        }),
      }))
      .filter((cat) => cat.checks.length > 0);

    return {
      ...report,
      categories: filteredCategories,
    };
  }, [report, localFilter]);

  // ── Derived stats ───────────────────────────────────────────────────────

  const totalChecks = report
    ? report.total_passed_count + report.total_failed_count + report.total_untested_count
    : 0;

  // ── AI Fix Handlers ─────────────────────────────────────────────────────

  const handleAIFix = useCallback(async (check: ComplianceCheckResult, category: string) => {
    if (!report || !agentInfo) return;

    setFixingCheck(check.title);
    setFixResult(null);
    setChatMessages([]);

    try {
      const input: FailedCheckInput = {
        title: check.title,
        remediation: check.remediation,
        os: report.os,
        mandatory: check.mandatory,
        category,
      };
      const result = await invoke<AiFixResult>("ai_fix_check", { input });
      setFixResult(result);
    } catch (e) {
      setFixResult({ markdown: "", success: false, error: String(e) });
    } finally {
      setFixingCheck(null);
    }
  }, [report, agentInfo]);

  const handleAIFixAll = useCallback(async () => {
    if (!report || !agentInfo) return;

    setFixingAll(true);
    setFixResult(null);

    const failedChecks: FailedCheckInput[] = [];
    for (const cat of report.categories) {
      for (const check of cat.checks) {
        if (check.status === "Failed") {
          failedChecks.push({
            title: check.title,
            remediation: check.remediation,
            os: report.os,
            mandatory: check.mandatory,
            category: cat.name,
          });
        }
      }
    }

    // Fix them one by one, collecting results
    const results: AiFixResult[] = [];
    for (const input of failedChecks) {
      try {
        const result = await invoke<AiFixResult>("ai_fix_check", { input });
        results.push(result);
      } catch (e) {
        results.push({ markdown: "", success: false, error: String(e) });
      }
    }

    // Combine all results into one markdown output
    const allMarkdown = results
      .filter((r) => r.success)
      .map((r, i) => `## ${i + 1}. ${failedChecks[i].title}\n\n${r.markdown}`)
      .join("\n\n---\n\n");

    const errors = results.filter((r) => !r.success);
    setFixResult({
      markdown: allMarkdown,
      success: errors.length === 0,
      error: errors.length > 0 ? `${errors.length} fix(es) failed` : null,
    });
    setFixingAll(false);
  }, [report, agentInfo]);

  // ── Follow-up Chat Handler ──────────────────────────────────────────────

  const handleChatSend = useCallback(async () => {
    const msg = chatInput.trim();
    if (!msg || !fixResult || chatSending) return;

    setChatInput("");
    setChatMessages((prev) => [...prev, { role: "user", content: msg }]);
    setChatSending(true);

    try {
      const reply = await invoke<string>("ai_chat", {
        prompt: msg,
        context: fixResult.markdown || null,
      });
      setChatMessages((prev) => [...prev, { role: "assistant", content: reply }]);
    } catch (e) {
      setChatMessages((prev) => [
        ...prev,
        { role: "assistant", content: `Error: ${e}` },
      ]);
    } finally {
      setChatSending(false);
    }
  }, [chatInput, chatSending, fixResult]);

  const closeFixResult = useCallback(() => {
    setFixResult(null);
    setFixingCheck(null);
    setChatMessages([]);
    setChatInput("");
    setCommandStates({});
    setSharedPassword("");
    setScaRescanState({ status: "idle", output: "" });
    setShowRefreshAfterRescan(false);
    setCopiedIndex(null);
  }, []);

  // Detect if a command needs sudo
  const commandNeedsSudo = (cmd: string) => /(?:^|\s)sudo\s/.test(cmd);

  // Detect interactive-only commands that can never run non-interactively
  const commandIsInteractive = (cmd: string) =>
    /\b(nano|vim?|emacs|gedit|kate|mousepad|xed)\b/.test(cmd);

  const handleRunCommand = useCallback(async (cmd: string, idx: number) => {
    setCommandStates((prev) => ({
      ...prev,
      [idx]: { status: "running", output: "Starting execution...\n" },
    }));

    try {
      let output: string;
      if (commandNeedsSudo(cmd)) {
        output = await invoke<string>("execute_fix_command_sudo", { command: cmd, sudoPassword: sharedPassword });
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
  }, [sharedPassword]);

  const handleSCARescan = useCallback(async () => {
    setScaRescanState({ status: "running", output: "Restarting wazuh-agent..." });
    setShowRefreshAfterRescan(false);
    try {
      const output = await invoke<string>("trigger_sca_rescan", { sudoPassword: sharedPassword });
      setScaRescanState({ status: "success", output });
      setShowRefreshAfterRescan(true);
    } catch (e) {
      setScaRescanState({ status: "failed", output: String(e) });
    }
  }, [sharedPassword]);

  const handleRefreshResults = useCallback(async () => {
    await fetchReport();
    setShowRefreshAfterRescan(false);
  }, [fetchReport]);

  // ── Loading ─────────────────────────────────────────────────────────────

  if (loading) {
    return (
      <div className="view-container">
        <div className="subtitle">Security Configuration Assessment</div>
        <h2 className="header title">System Compliance</h2>
        <div className="skeleton" style={{ height: "140px", marginBottom: "16px" }} />
        <div className="skeleton" style={{ height: "80px", marginBottom: "16px" }} />
        <div className="skeleton" style={{ height: "60px", marginBottom: "8px" }} />
        <div className="skeleton" style={{ height: "60px", marginBottom: "8px" }} />
        <div className="skeleton" style={{ height: "60px", marginBottom: "8px" }} />
      </div>
    );
  }

  // ── Error ───────────────────────────────────────────────────────────────

  if (error) {
    return (
      <div className="view-container">
        <div className="subtitle">Security Configuration Assessment</div>
        <h2 className="header title">System Compliance</h2>
        <div className="compliance-error">
          <div className="compliance-error-icon">!</div>
          <div className="compliance-error-title">Failed to load compliance data</div>
          <div className="compliance-error-text">{error}</div>
          <button className="update-button" onClick={fetchReport}>
            Retry
          </button>
        </div>
      </div>
    );
  }

  if (!report || !agentInfo) return null;

  const isCompliant = report.compliance_status === "Passed";
  const circ = 2 * Math.PI * 52; // circumference for r=52
  const scoreColorVal = scoreColor(report.score);
  const scoreLabelText = scoreLabel(report.score);
  // Start at empty (offset = circ), animate to target
  const targetOffset = circ - (report.score / 100) * circ;
  const offset = animateScore ? targetOffset : circ;

  const failedCount = report.total_failed_count;

  return (
    <div className="view-container">
      {/* Header */}
      <div className="subtitle">
        Security Configuration Assessment
        {lastUpdated && (
          <span className="compliance-last-updated">
            {" · "}Updated {formatRelativeTime(lastUpdated)}
          </span>
        )}
      </div>
      <div className="header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h2 className="title">System Compliance</h2>
        <button className="compliance-refresh-btn" onClick={fetchReport} title="Refresh now">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
          </svg>
        </button>
      </div>

      {/* ── Hero Score Card ────────────────────────────────────────────── */}
      <div className="compliance-hero">
        <div className="compliance-score-block">
          <div className="compliance-score-ring" style={{ "--bubble-color": scoreColorVal } as CSSProperties}>
            <svg width="96" height="96" viewBox="0 0 128 128">
              <circle cx="64" cy="64" r="52" fill="none" stroke="var(--border)" strokeWidth="8" />
              <circle
                cx="64" cy="64" r="52"
                fill="none"
                stroke={scoreColorVal}
                strokeWidth="8"
                strokeLinecap="round"
                strokeDasharray={circ}
                strokeDashoffset={offset}
                transform="rotate(-90 64 64)"
                className="compliance-score-arc"
              />
            </svg>
            <div className="compliance-score-value" style={{ color: scoreColorVal }}>
              {report.score}
              <span className="compliance-score-unit">%</span>
            </div>
          </div>
          <div className="compliance-score-badge" style={{ background: `${scoreColorVal}18`, color: scoreColorVal, borderColor: `${scoreColorVal}30` }}>
            {scoreLabelText}
          </div>
        </div>

        <div className="compliance-hero-details">
          <div className="compliance-hero-top">
            <div className="compliance-agent-icon">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
                <line x1="8" y1="21" x2="16" y2="21" />
                <line x1="12" y1="17" x2="12" y2="21" />
              </svg>
            </div>
            <div className={`compliance-compliance-badge ${isCompliant ? "pass" : "fail"}`}>
              <span className="compliance-badge-dot" />
              {isCompliant ? "Compliant" : "Non-Compliant"}
            </div>
          </div>
          <div className="compliance-agent-text">
            <div className="compliance-agent-name">{agentInfo.name}</div>
            <div className="compliance-agent-id">ID: {agentInfo.id} · {report.os}</div>
          </div>
        </div>
      </div>

      {/* ── Summary Grid (vertical layout - no overflow) ────────────── */}
      <div className="compliance-summary-grid">
        <div className="compliance-stat-card" style={{ animationDelay: "0.05s" }}>
          <div className="compliance-stat-icon pass-bg">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </div>
          <div className="compliance-stat-number pass">{report.total_passed_count}</div>
          <div className="compliance-stat-label">Passed</div>
        </div>
        <div className="compliance-stat-card" style={{ animationDelay: "0.1s" }}>
          <div className="compliance-stat-icon fail-bg">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </div>
          <div className="compliance-stat-number fail">{report.total_failed_count}</div>
          <div className="compliance-stat-label">Failed</div>
        </div>
        {report.total_untested_count > 0 && (
          <div className="compliance-stat-card" style={{ animationDelay: "0.15s" }}>
            <div className="compliance-stat-icon dim-bg">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <line x1="12" y1="5" x2="12" y2="19" />
              </svg>
            </div>
            <div className="compliance-stat-number untested">{report.total_untested_count}</div>
            <div className="compliance-stat-label">Untested</div>
          </div>
        )}
        <div className="compliance-stat-card" style={{ animationDelay: "0.2s" }}>
          <div className="compliance-stat-icon total-bg">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
          </div>
          <div className="compliance-stat-number total-count">{totalChecks}</div>
          <div className="compliance-stat-label">Total</div>
        </div>
      </div>

      {/* ── Categories ────────────────────────────────────────────────── */}
      <div className="compliance-categories-header">
        <div className="section-title" style={{ margin: 0 }}>Categories</div>

        {/* Client-side filter pills */}
        <div className="compliance-filter-row">
          {(["all", "passed", "failed"] as const).map((val) => (
            <button
              key={val}
              className={`compliance-filter-chip ${localFilter === val ? "active" : ""}`}
              onClick={() => setLocalFilter(val)}
            >
              {val === "all" ? "All" : val.charAt(0).toUpperCase() + val.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* ── AI Fix All Button ──────────────────────────────────────────── */}
      {aiStatus?.configured && failedCount > 0 && (
        <div style={{ marginBottom: "10px", display: "flex", alignItems: "center", gap: "8px" }}>
          <button
            className="update-button"
            style={{ fontSize: "0.75rem", padding: "6px 14px", display: "flex", alignItems: "center", gap: "6px" }}
            onClick={handleAIFixAll}
            disabled={fixingAll}
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 2l2.4 7.2H22l-6 4.8 2.4 7.2L12 16l-6 4.8L8.4 14l-6-4.8h7.6z" />
            </svg>
            {fixingAll ? "Fixing all..." : `Fix All ${failedCount} Failed`}
          </button>
          {fixingAll && (
            <span className="compliance-last-updated" style={{ fontSize: "0.75rem" }}>Processing, please wait...</span>
          )}
        </div>
      )}

      {/* ── AI Not Configured Banner ───────────────────────────────────── */}
      {!aiStatus?.configured && failedCount > 0 && (
        <div style={{ marginBottom: "10px", padding: "8px 12px", fontSize: "0.75rem", color: "var(--text-dim)", background: "var(--card-bg)", borderRadius: "8px", border: "1px solid var(--border)" }}>
          <span style={{ marginRight: "6px" }}>💡</span>
          Configure an AI provider in Settings to get AI-powered fix suggestions for failed checks.
        </div>
      )}

      <div className="compliance-categories">
        {filteredReport && filteredReport.categories.length === 0 ? (
          <div className="compliance-empty-state">
            No {localFilter === "all" ? "" : localFilter} checks in any category.
          </div>
        ) : (
          filteredReport?.categories.map((cat) => {
            const isExpanded = expandedCategory === cat.name;
            const total = cat.passed_count + cat.failed_count + cat.untested_count;
            const passPct = total > 0 ? Math.round((cat.passed_count / total) * 100) : 0;

            return (
              <div key={cat.name} className={`compliance-category ${isExpanded ? "expanded" : ""}`}>
                <button className="compliance-category-trigger" onClick={() =>
                  setExpandedCategory(isExpanded ? null : cat.name)
                }>
                  <div className="compliance-category-summary">
                    <div className="compliance-category-info">
                      <div className="compliance-category-name">{cat.name}</div>
                      <div className="compliance-category-meta">
                        <span className="pass">{cat.passed_count} passed</span>
                        {cat.failed_count > 0 && (
                          <><span className="sep">·</span><span className="fail">{cat.failed_count} failed</span></>
                        )}
                        {cat.untested_count > 0 && (
                          <><span className="sep">·</span><span className="dim">{cat.untested_count} untested</span></>
                        )}
                      </div>
                    </div>
                    <div className="compliance-category-progress">
                      <div className="compliance-progress-bar">
                        <div
                          className="compliance-progress-fill"
                          style={{ width: `${passPct}%` }}
                        />
                      </div>
                      <span className="compliance-progress-label">{passPct}%</span>
                    </div>
                  </div>
                  <svg
                    width="16" height="16" viewBox="0 0 24 24"
                    fill="none" stroke="currentColor" strokeWidth="2"
                    strokeLinecap="round" strokeLinejoin="round"
                    className={`compliance-chevron ${isExpanded ? "open" : ""}`}
                  >
                    <polyline points="6 9 12 15 18 9" />
                  </svg>
                </button>

                {isExpanded && (
                  <div className="compliance-category-body">
                    {cat.checks.map((check) => (
                      <ComplianceCheckRow
                        key={check.check_id}
                        check={check}
                        aiConfigured={aiStatus?.configured ?? false}
                        onFix={() => handleAIFix(check, cat.name)}
                        fixing={fixingCheck === check.title}
                      />
                    ))}
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>

      {/* ── AI Fix Result Modal + Follow-up Chat ───────────────────────── */}
      {fixResult && (
        <div className="update-modal-backdrop" onClick={closeFixResult}>
          <div className="ai-fix-modal" onClick={(e) => e.stopPropagation()}>
            <div className="update-modal-header">
              <div className="update-modal-title">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" style={{ color: "var(--primary)" }}>
                  <path d="M12 2l2.4 7.2H22l-6 4.8 2.4 7.2L12 16l-6 4.8L8.4 14l-6-4.8h7.6z" />
                </svg>
                {fixResult.success ? "AI Fix Suggestions" : "Fix Generation Failed"}
              </div>
              <button onClick={closeFixResult} className="compliance-refresh-btn">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>

            {fixResult.success ? (
              <div className="ai-fix-modal-panes">
                {/* Left Pane: Fix steps & Interactive Command Executions */}
                <div className="ai-fix-modal-left">
                  {/* Risk & Impact Alert Card */}
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
                        <div className="compliance-fix-impact-desc">
                          {parsedData.impact}
                        </div>
                      )}
                    </div>
                  )}

                  <div className="ai-fix-markdown">
                    {parsedData.chunks.map((chunk, i) => {
                      if (chunk.type === "heading2") {
                        return <h3 key={i} className="ai-fix-heading">{chunk.content}</h3>;
                      }
                      if (chunk.type === "heading3") {
                        return <h4 key={i} className="ai-fix-subheading">{chunk.content}</h4>;
                      }
                      if (chunk.type === "step") {
                        return <div key={i} className="ai-fix-step">{chunk.content}</div>;
                      }
                      if (chunk.type === "list_item") {
                        return <li key={i} className="ai-fix-list-item">{chunk.content}</li>;
                      }
                      if (chunk.type === "text") {
                        return <p key={i} className="ai-fix-paragraph">{chunk.content}</p>;
                      }
                      if (chunk.type === "code_block") {
                        const execState = commandStates[i] || { status: "idle", output: "" };
                        return (
                          <div key={i} className={`compliance-command-card ${execState.status}`}>
                            <div className="compliance-command-card-header">
                              <span className="compliance-command-title">
                                Suggested Shell Command
                              </span>
                              <div className="compliance-command-actions">
                                <button
                                  className={`ai-copy-btn ${copiedIndex === i ? "copied" : ""}`}
                                  onClick={() => {
                                    navigator.clipboard.writeText(chunk.content);
                                    setCopiedIndex(i);
                                    setTimeout(() => setCopiedIndex((prev) => prev === i ? null : prev), 1500);
                                  }}
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

                            {/* Interactive editor warning */}
                            {commandIsInteractive(chunk.content) && (
                              <div className="compliance-command-interactive-warn">
                                ⚠ This command opens an interactive editor and cannot run inside the app. Copy it and run it in a terminal.
                              </div>
                            )}

                            <pre className="compliance-command-text">
                              <code>{chunk.content}</code>
                            </pre>

                            {/* Sudo password row */}
                            {commandNeedsSudo(chunk.content) && !commandIsInteractive(chunk.content) && (
                              <div className="compliance-sudo-row">
                                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{ flexShrink: 0, opacity: 0.5 }}>
                                  <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                                  <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                                </svg>
                                <input
                                  type="password"
                                  className="compliance-sudo-input"
                                  placeholder="sudo password"
                                  value={sharedPassword}
                                  onChange={(e) => setSharedPassword(e.target.value)}
                                  spellCheck={false}
                                  autoComplete="current-password"
                                />
                              </div>
                            )}

                            {execState.output && (
                              <div className={`compliance-command-terminal ${execState.status}`}>
                                <div className="compliance-command-terminal-header">
                                  <span>Console Output</span>
                                  <span className={`terminal-status-dot ${execState.status}`} />
                                </div>
                                <pre className="compliance-command-terminal-log">
                                  {execState.output}
                                </pre>
                              </div>
                            )}
                          </div>
                        );
                      }
                      return null;
                    })}
                  </div>

                  {/* SCA Rescan Card */}
                  <div className="sca-rescan-card">
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
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" style={{ flexShrink: 0, opacity: 0.5 }}>
                        <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                        <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                      </svg>
                      <input
                        type="password"
                        className="compliance-sudo-input"
                        placeholder="sudo password"
                        value={sharedPassword}
                        onChange={(e) => setSharedPassword(e.target.value)}
                        spellCheck={false}
                        autoComplete="current-password"
                      />
                      <button
                        className="compliance-command-run-btn"
                        onClick={handleSCARescan}
                        disabled={scaRescanState.status === "running" || !sharedPassword.trim()}
                      >
                        {scaRescanState.status === "running" ? (
                          <><span className="settings-ai-spinner" /> Restarting...</>
                        ) : (
                          <>
                            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                              <polygon points="5 3 19 12 5 21 5 3" />
                            </svg>
                            Restart &amp; Rescan
                          </>
                        )}
                      </button>
                    </div>

                    {scaRescanState.output && (
                      <div className={`compliance-command-terminal ${scaRescanState.status}`}>
                        <div className="compliance-command-terminal-header">
                          <span>Restart Output</span>
                          <span className={`terminal-status-dot ${scaRescanState.status}`} />
                        </div>
                        <pre className="compliance-command-terminal-log">{scaRescanState.output}</pre>
                      </div>
                    )}

                    {showRefreshAfterRescan && (
                      <div className="sca-rescan-refresh-hint">
                        <span>SCA scan started. Results typically appear within 30–60 seconds.</span>
                        <button className="compliance-verify-btn" onClick={handleRefreshResults}>
                          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                            <path d="M23 4v6h-6" />
                            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
                          </svg>
                          Refresh Results
                        </button>
                      </div>
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
                      {chatMessages.map((m, i) => (
                        <div key={i} className={`ai-chat-msg ${m.role}`}>
                          <div className="ai-chat-msg-role">
                            {m.role === "user" ? "You" : "AI"}
                          </div>
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

            <button className="update-modal-dismiss" onClick={closeFixResult}>
              Close
            </button>
          </div>
        </div>
      )}

    </div>
  );
}

// ─── Check Row ────────────────────────────────────────────────────────────────

function ComplianceCheckRow({
  check,
  aiConfigured,
  onFix,
  fixing,
}: {
  check: ComplianceCheckResult;
  aiConfigured: boolean;
  onFix: () => void;
  fixing: boolean;
}) {
  const isPassed = check.status === "Passed";
  const isFailed = check.status === "Failed";

  return (
    <div className={`compliance-check ${isFailed ? "is-failed" : ""}`}>
      <div className={`compliance-check-icon ${isPassed ? "pass" : isFailed ? "fail" : "dim"}`}>
        {isPassed ? (
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        ) : isFailed ? (
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
            <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        ) : (
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
            <line x1="12" y1="5" x2="12" y2="19" />
          </svg>
        )}
      </div>

      <div className="compliance-check-body">
        <div className="compliance-check-title">
          {check.title}
          {check.mandatory && <span className="compliance-badge">Required</span>}
        </div>

        {isFailed && check.remediation && (
          <div className="compliance-remediation">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="16" x2="12" y2="12" />
              <line x1="12" y1="8" x2="12.01" y2="8" />
            </svg>
            <span>{check.remediation}</span>
          </div>
        )}

        {/* AI Fix button for failed checks */}
        {isFailed && aiConfigured && (
          <button
            className="ai-fix-btn"
            onClick={onFix}
            disabled={fixing}
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 2l2.4 7.2H22l-6 4.8 2.4 7.2L12 16l-6 4.8L8.4 14l-6-4.8h7.6z" />
            </svg>
            {fixing ? "Generating fix..." : "Fix with AI"}
          </button>
        )}
      </div>
    </div>
  );
}

// ─── Relative time helper ─────────────────────────────────────────────────────

function formatRelativeTime(date: Date): string {
  const diff = Date.now() - date.getTime();
  const secs = Math.floor(diff / 1000);
  if (secs < 5) return "just now";
  if (secs < 60) return `${secs}s ago`;
  const mins = Math.floor(secs / 60);
  if (mins === 1) return "1m ago";
  return `${mins}m ago`;
}
