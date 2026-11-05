import { describe, it, expect } from "vitest";
import { isCompliantStatus } from "./compliance";

describe("isCompliantStatus", () => {
  it("accepts the gateway's compliant status", () => {
    expect(isCompliantStatus("compliant")).toBe(true);
    expect(isCompliantStatus("Compliant")).toBe(true);
  });

  it("rejects non-compliant", () => {
    expect(isCompliantStatus("non-compliant")).toBe(false);
    expect(isCompliantStatus("Non-Compliant")).toBe(false);
  });

  it("keeps accepting the legacy Passed/Failed values", () => {
    expect(isCompliantStatus("Passed")).toBe(true);
    expect(isCompliantStatus("passed")).toBe(true);
    expect(isCompliantStatus("Failed")).toBe(false);
  });

  it("treats missing or unknown status as not compliant", () => {
    expect(isCompliantStatus("")).toBe(false);
    expect(isCompliantStatus(undefined)).toBe(false);
    expect(isCompliantStatus(null)).toBe(false);
    expect(isCompliantStatus("unknown")).toBe(false);
  });
});
