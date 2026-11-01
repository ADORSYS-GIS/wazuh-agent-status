import type { ComplianceReport } from "../types/agent";

// ─── OS update / restart reminders (ADR-0003) ─────────────────────────────────
//
// The gateway reports two company checks per OS: "no OS updates pending" and
// "restarted recently". When either fails, the user is reminded (desktop
// notification where supported, in-app banner everywhere) at most once a day.

export type OsReminderKind = "update" | "restart";

export interface OsReminder {
  kind: OsReminderKind;
  title: string;
  body: string;
  remediation: string;
  /** Days left before the device turns non-compliant; undefined once overdue. */
  graceRemainingDays?: number;
}

// Matched by title, which is stable across OSes (check IDs differ per OS).
const CHECK_TITLES: Record<OsReminderKind, string> = {
  update: "Ensure no OS updates are pending",
  restart: "Ensure the system has been restarted recently",
};

function normalize(title: string): string {
  return title.replace(/[^a-z0-9]/gi, "").toLowerCase();
}

function graceText(days: number | undefined): string {
  if (days === undefined) return "Your device is now non-compliant until this is done.";
  if (days === 1) return "1 day left before your device becomes non-compliant.";
  return `${days} days left before your device becomes non-compliant.`;
}

/** Builds the reminders for the failing update/restart checks of a report. */
export function getOsReminders(report: ComplianceReport | null | undefined): OsReminder[] {
  if (!report) return [];
  const checks = report.categories.flatMap((c) => c.checks);
  const reminders: OsReminder[] = [];

  (Object.keys(CHECK_TITLES) as OsReminderKind[]).forEach((kind) => {
    const check = checks.find(
      (c) => normalize(c.title) === normalize(CHECK_TITLES[kind]) && c.status === "Failed",
    );
    if (!check) return;
    const days = check.grace_remaining_days;
    reminders.push({
      kind,
      title: kind === "update" ? "Updates are available" : "Please restart your laptop",
      body:
        (kind === "update"
          ? "Install the pending system updates to keep your laptop secure. "
          : "Restart your laptop so installed updates take effect. ") + graceText(days),
      remediation: check.remediation,
      graceRemainingDays: days,
    });
  });

  return reminders;
}

/** Local calendar day as YYYY-MM-DD, used to send each reminder once a day. */
export function dayKey(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

const STORAGE_PREFIX = "wazuh_os_reminder_";

/**
 * Returns true (and records today) if the reminder of this kind has not been
 * sent yet today.
 */
export function claimDailyReminder(
  kind: OsReminderKind,
  now: Date,
  storage: Pick<Storage, "getItem" | "setItem"> = localStorage,
): boolean {
  const key = STORAGE_PREFIX + kind;
  const today = dayKey(now);
  if (storage.getItem(key) === today) return false;
  storage.setItem(key, today);
  return true;
}
