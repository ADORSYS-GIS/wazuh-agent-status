import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { OsReminderBanner } from "./OsReminderBanner";
import type { OsReminder } from "../utils/osReminders";

const update: OsReminder = {
  kind: "update",
  title: "Updates are available",
  body: "Install the pending system updates. 5 days left before your device becomes non-compliant.",
  remediation: "System Settings -> Software Update",
  graceRemainingDays: 5,
};

describe("OsReminderBanner", () => {
  it("renders nothing without reminders", () => {
    const { container } = render(<OsReminderBanner reminders={[]} onOpenCompliance={() => {}} />);
    expect(container.firstChild).toBeNull();
  });

  it("shows the reminder and opens the compliance view", () => {
    const onOpen = vi.fn();
    render(<OsReminderBanner reminders={[update]} onOpenCompliance={onOpen} />);
    expect(screen.getByText("Updates are available")).toBeTruthy();
    fireEvent.click(screen.getByText("How to fix"));
    expect(onOpen).toHaveBeenCalledOnce();
  });

  it("can be dismissed", () => {
    render(<OsReminderBanner reminders={[update]} onOpenCompliance={() => {}} />);
    fireEvent.click(screen.getByLabelText("Dismiss"));
    expect(screen.queryByText("Updates are available")).toBeNull();
  });
});
