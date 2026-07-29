import { useState, useEffect, useCallback, useMemo, useRef, type CSSProperties } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AgentStatus, ComplianceReport, ComplianceCheckResult } from "../types/agent";
import type { AiFixResult, AiProviderStatus, FailedCheckInput } from "../types/ai";
import { ComplianceFixModal } from "./ComplianceFixModal";
import { scoreColor, scoreLabel, formatRelativeTime } from "../utils/compliance";

// ─── Sub-Components ──────────────────────────────────────────────────────────

function ComplianceSkeleton() {
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

interface ComplianceErrorProps {
  error: string;
  onRetry: () => void;
}

function ComplianceError({ error, onRetry }: Readonly<ComplianceErrorProps>) {
  return (
    <div className="view-container">
      <div className="subtitle">Security Configuration Assessment</div>
      <h2 className="header title">System Compliance</h2>
      <div className="compliance-error">
        <div className="compliance-error-icon">!</div>
        <div className="compliance-error-title">Failed to load compliance data</div>
        <div className="compliance-error-text">{error}</div>
        <button type="button" className="compliance-retry-btn" onClick={onRetry}>
          Retry
        </button>
      </div>
    </div>
  );
}

// ─── Main Component ───────────────────────────────────────────────────────────

export function ComplianceView({ agentStatus }: Readonly<{ agentStatus: AgentStatus }>) {
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

  // ── SCA Rescan State ────────────────────────────────────────────────────
  const [scaRescanState, setScaRescanState] = useState<{ status: "idle" | "running" | "success" | "failed"; output: string }>({ status: "idle", output: "" });
  const scaTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Clean up the rescan auto-clear timeout on unmount
  useEffect(() => {
    return () => {
      if (scaTimeoutRef.current) {
        clearTimeout(scaTimeoutRef.current);
      }
    };
  }, []);

  const handleSCARescan = useCallback(async () => {
    // Cancel any pending auto-clear from a previous rescan
    if (scaTimeoutRef.current) {
      clearTimeout(scaTimeoutRef.current);
    }
    setScaRescanState({ status: "running", output: "Restarting wazuh-agent to trigger SCA rescan..." });
    try {
      const output = await invoke<string>("trigger_sca_rescan");
      setScaRescanState({ status: "success", output });
      // Auto-clear the output after 4 seconds so it doesn't linger
      scaTimeoutRef.current = setTimeout(() => {
        setScaRescanState({ status: "idle", output: "" });
        scaTimeoutRef.current = null;
      }, 4000);
    } catch (e) {
      setScaRescanState({ status: "failed", output: String(e) });
    }
  }, []);

  // ── AI Fix State ──────────────────────────────────────────────────────────
  const [aiStatus, setAiStatus] = useState<AiProviderStatus | null>(null);
  const [fixingCheck, setFixingCheck] = useState<string | null>(null); // check title being fixed
  const [fixResult, setFixResult] = useState<AiFixResult | null>(null);
  const [fixingAll, setFixingAll] = useState(false);

  // Load AI provider status (never fails — returns { configured: false } if unset)
  useEffect(() => {
    invoke<AiProviderStatus>("get_ai_status").then(setAiStatus);
  }, []);

  const precheckError = useMemo<string | null>(() => {
    if (!agentStatus.agent_id) {
      return "No agent enrollment found. This machine does not appear to be registered with any Wazuh Manager. Please enroll the agent first.";
    }
    if (agentStatus.status !== "Active") {
      return "Wazuh agent is not running on this machine. Please start the agent service and try again.";
    }
    if (agentStatus.connection !== "Connected") {
      return "Agent is not connected to the Wazuh Manager. The agent was found but has no active connection \u2014 SCA results are unavailable.";
    }
    return null;
  }, [agentStatus.agent_id, agentStatus.status, agentStatus.connection]);

  const fetchReport = useCallback(async () => {
    setLoading(true);

    if (precheckError) {
      setError(precheckError);
      setLoading(false);
      return;
    }

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
  }, [agentStatus.agent_id, agentStatus.agent_name, precheckError]);

  useEffect(() => {
    if (agentStatus.agent_id) {
      fetchReport();
    } else {
      setLoading(false);
    }
  }, [fetchReport, agentStatus.agent_id]);

  // Auto-refresh the "Updated X ago" timestamp every 10 seconds
  useEffect(() => {
    if (!lastUpdated) return;
    const timer = setInterval(() => {
      setLastUpdated(new Date());
    }, 10_000);
    return () => clearInterval(timer);
  }, [lastUpdated]);

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

    try {
      const results = await invoke<AiFixResult[]>("ai_fix_batch", { inputs: failedChecks });
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
    } catch (e) {
      setFixResult({ markdown: "", success: false, error: String(e) });
    } finally {
      setFixingAll(false);
    }
  }, [report, agentInfo]);

  const closeFixResult = useCallback(() => {
    setFixResult(null);
    setFixingCheck(null);
  }, []);

  // ── Loading ─────────────────────────────────────────────────────────────

  if (loading) {
    return <ComplianceSkeleton />;
  }

  // ── Error ───────────────────────────────────────────────────────────────

  if (error) {
    return <ComplianceError error={error} onRetry={fetchReport} />;
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
        <div style={{ display: "flex", gap: "6px" }}>
          <button type="button" className="compliance-refresh-btn" onClick={() => { closeFixResult(); fetchReport(); }} title="Refresh now">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="23 4 23 10 17 10" />
              <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
            </svg>
          </button>
          <button
            className="compliance-refresh-btn"
            onClick={handleSCARescan}
            disabled={scaRescanState.status === "running"}
            title="Restart agent to trigger SCA rescan"
            style={{ color: scaRescanState.status === "running" ? "var(--primary)" : undefined }}
          >
            {scaRescanState.status === "running" ? (
              <span className="settings-ai-spinner" style={{ width: 12, height: 12 }} />
            ) : (
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                <polyline points="23 4 23 10 17 10" />
                <polyline points="1 20 1 14 7 14" />
                <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
              </svg>
            )}
          </button>
        </div>
      </div>

      {/* ── SCA Rescan Terminal Output ───────────────────────────────── */}
      {scaRescanState.output && (
        <div className={`compliance-command-terminal ${scaRescanState.status}`} style={{ marginBottom: "18px" }}>
          <div className="compliance-command-terminal-header">
            <span>SCA Rescan</span>
            <span className={`terminal-status-dot ${scaRescanState.status}`} />
          </div>
          <pre className="compliance-command-terminal-log">{scaRescanState.output}</pre>
        </div>
      )}

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
              type="button"
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
            type="button"
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
          <span>Configure an AI provider in Settings to get AI-powered fix suggestions for failed checks.</span>
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
                <button type="button" className="compliance-category-trigger" onClick={() =>
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
        <ComplianceFixModal
          fixResult={fixResult}
          onClose={closeFixResult}
        />
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
}: Readonly<{
  check: ComplianceCheckResult;
  aiConfigured: boolean;
  onFix: () => void;
  fixing: boolean;
}>) {
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
            type="button"
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


