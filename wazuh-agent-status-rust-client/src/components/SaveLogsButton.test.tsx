import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import "@testing-library/jest-dom/vitest";
import { SaveLogsButton } from "./SaveLogsButton";

describe("SaveLogsButton", () => {
  it("shows 'Save Logs' and is enabled when state is idle", () => {
    render(<SaveLogsButton downloadState="idle" onSave={vi.fn()} title="Save" />);
    const btn = screen.getByRole("button");
    expect(btn).toHaveTextContent("Save Logs");
    expect(btn).not.toBeDisabled();
  });

  it("shows 'Saving…' and is disabled when state is loading", () => {
    render(<SaveLogsButton downloadState="loading" onSave={vi.fn()} title="Save" />);
    const btn = screen.getByRole("button");
    expect(btn).toHaveTextContent("Saving…");
    expect(btn).toBeDisabled();
  });

  it("shows 'Saved' when state is success", () => {
    render(<SaveLogsButton downloadState="success" onSave={vi.fn()} title="Save" />);
    expect(screen.getByRole("button")).toHaveTextContent("Saved");
  });

  it("applies state as CSS class", () => {
    const { rerender } = render(
      <SaveLogsButton downloadState="idle" onSave={vi.fn()} title="Save" />
    );
    expect(screen.getByRole("button").className).toContain("idle");

    rerender(<SaveLogsButton downloadState="error" onSave={vi.fn()} title="Save" />);
    expect(screen.getByRole("button").className).toContain("error");
  });

  it("calls onSave when clicked in idle state", () => {
    const onSave = vi.fn();
    render(<SaveLogsButton downloadState="idle" onSave={onSave} title="Save" />);
    fireEvent.click(screen.getByRole("button"));
    expect(onSave).toHaveBeenCalledOnce();
  });

  it("does NOT call onSave when clicked in loading state", () => {
    const onSave = vi.fn();
    render(<SaveLogsButton downloadState="loading" onSave={onSave} title="Save" />);
    fireEvent.click(screen.getByRole("button"));
    expect(onSave).not.toHaveBeenCalled();
  });

  it("forwards extra className to the button element", () => {
    render(
      <SaveLogsButton downloadState="idle" onSave={vi.fn()} title="Save" className="extra" />
    );
    expect(screen.getByRole("button").className).toContain("extra");
  });
});
