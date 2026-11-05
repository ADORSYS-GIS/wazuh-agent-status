import { useState } from "react";
import type { OsReminder } from "../utils/osReminders";

// In-app reminder to update / restart (ADR-0003). Shown on every OS; on macOS
// it is the only reminder until the client ships as a Developer ID–signed app
// that can post native notifications.
export function OsReminderBanner({
  reminders,
  onOpenCompliance,
}: Readonly<{ reminders: OsReminder[]; onOpenCompliance: () => void }>) {
  const [dismissed, setDismissed] = useState<Set<string>>(new Set());
  const visible = reminders.filter((r) => !dismissed.has(r.kind));
  if (visible.length === 0) return null;

  return (
    <div role="status" aria-live="polite" style={{ display: "flex", flexDirection: "column", gap: "8px", margin: "12px 20px 0" }}>
      {visible.map((r) => {
        const overdue = r.graceRemainingDays === undefined;
        const accent = overdue ? "var(--error)" : "var(--warning)";
        return (
          <div
            key={r.kind}
            style={{
              display: "flex",
              alignItems: "center",
              gap: "12px",
              padding: "10px 12px",
              background: "var(--card-bg)",
              border: `1px solid ${accent}`,
              borderRadius: "8px",
              fontSize: "0.8rem",
            }}
          >
            <span aria-hidden="true">{r.kind === "update" ? "⬆️" : "🔄"}</span>
            <div style={{ flex: 1 }}>
              <div style={{ fontWeight: 600, color: accent }}>{r.title}</div>
              <div style={{ color: "var(--text-dim)" }}>{r.body}</div>
            </div>
            <button type="button" className="settings-ai-btn settings-ai-btn-outline" onClick={onOpenCompliance}>
              How to fix
            </button>
            <button
              type="button"
              aria-label="Dismiss"
              title="Dismiss until the next check"
              onClick={() => setDismissed((d) => new Set(d).add(r.kind))}
              style={{ background: "none", border: "none", color: "var(--text-dim)", cursor: "pointer" }}
            >
              ✕
            </button>
          </div>
        );
      })}
    </div>
  );
}
