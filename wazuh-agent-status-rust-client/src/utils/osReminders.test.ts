import { describe, it, expect } from "vitest";
import type { ComplianceReport, ComplianceCheckResult } from "../types/agent";
import { getOsReminders, claimDailyReminder, dayKey } from "./osReminders";

function report(checks: Partial<ComplianceCheckResult>[]): ComplianceReport {
  return {
    agent_id: "038",
    agent_name: "laptop",
    os: "macOS",
    score: 90,
    compliance_status: "compliant",
    total_passed_count: 0,
    total_failed_count: 0,
    total_untested_count: 0,
    categories: [
      {
        name: "Operating system and updates",
        status: "Failed",
        passed_count: 0,
        failed_count: 0,
        untested_count: 0,
        checks: checks.map((c, i) => ({
          check_id: i,
          title: "",
          status: "Passed",
          mandatory: true,
          remediation: "do it",
          ...c,
        })),
      },
    ],
  };
}

describe("getOsReminders", () => {
  it("returns nothing for a missing report or passing checks", () => {
    expect(getOsReminders(null)).toEqual([]);
    expect(
      getOsReminders(
        report([
          { title: "Ensure no OS updates are pending", status: "Passed" },
          { title: "Ensure the system has been restarted recently", status: "Passed" },
        ]),
      ),
    ).toEqual([]);
  });

  it("reminds about updates with the remaining grace days", () => {
    const [r] = getOsReminders(
      report([{ title: "Ensure no OS updates are pending", status: "Failed", grace_remaining_days: 5 }]),
    );
    expect(r.kind).toBe("update");
    expect(r.title).toBe("Updates are available");
    expect(r.graceRemainingDays).toBe(5);
    expect(r.body).toContain("5 days left");
  });

  it("reminds about restart and says when the grace period is over", () => {
    const [r] = getOsReminders(
      report([{ title: "Ensure the system has been restarted recently", status: "Failed" }]),
    );
    expect(r.kind).toBe("restart");
    expect(r.title).toBe("Please restart your laptop");
    expect(r.graceRemainingDays).toBeUndefined();
    expect(r.body).toContain("now non-compliant");
  });

  it("uses singular for one day and ignores untested checks", () => {
    const reminders = getOsReminders(
      report([
        { title: "Ensure no OS updates are pending", status: "Failed", grace_remaining_days: 1 },
        { title: "Ensure the system has been restarted recently", status: "Untested" },
      ]),
    );
    expect(reminders).toHaveLength(1);
    expect(reminders[0].body).toContain("1 day left");
  });
});

describe("claimDailyReminder", () => {
  function memoryStorage() {
    const m = new Map<string, string>();
    return { getItem: (k: string) => m.get(k) ?? null, setItem: (k: string, v: string) => void m.set(k, v) };
  }

  it("allows one reminder per kind per day", () => {
    const s = memoryStorage();
    const day1 = new Date(2026, 10, 1, 9, 0);
    expect(claimDailyReminder("update", day1, s)).toBe(true);
    expect(claimDailyReminder("update", new Date(2026, 10, 1, 18, 0), s)).toBe(false);
    expect(claimDailyReminder("restart", day1, s)).toBe(true);
    expect(claimDailyReminder("update", new Date(2026, 10, 2, 9, 0), s)).toBe(true);
  });

  it("formats the local day", () => {
    expect(dayKey(new Date(2026, 0, 5))).toBe("2026-01-05");
  });
});
