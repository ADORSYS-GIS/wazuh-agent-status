import { useState, useEffect, useCallback, useMemo, type CSSProperties } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AgentStatus, ComplianceReport, ComplianceCheckResult } from "../types/agent";

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
            <svg width="80" height="80" viewBox="0 0 128 128">
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
                      <ComplianceCheckRow key={check.check_id} check={check} />
                    ))}
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>


    </div>
  );
}

// ─── Check Row ────────────────────────────────────────────────────────────────

function ComplianceCheckRow({ check }: { check: ComplianceCheckResult }) {
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
