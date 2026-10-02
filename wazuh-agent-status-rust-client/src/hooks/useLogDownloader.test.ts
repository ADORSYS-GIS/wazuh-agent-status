import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { useLogDownloader } from "./useLogDownloader";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

/** Helper: run downloadLogs and wait for the state update to settle */
async function runDownload(
  result: { current: ReturnType<typeof useLogDownloader> },
  content = "content",
  prefix = "ossec"
) {
  await act(async () => {
    await result.current.downloadLogs(content, prefix);
  });
}

describe("useLogDownloader", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    mockInvoke.mockReset();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("starts with idle state and null message", () => {
    const { result } = renderHook(() => useLogDownloader());
    expect(result.current.saveState).toBe("idle");
    expect(result.current.saveMsg).toBeNull();
  });

  it("transitions to loading immediately on call", () => {
    mockInvoke.mockReturnValue(new Promise(() => {}));
    const { result } = renderHook(() => useLogDownloader());

    act(() => { result.current.downloadLogs("content", "ossec"); });

    expect(result.current.saveState).toBe("loading");
  });

  it("transitions to success with the saved path", async () => {
    mockInvoke.mockResolvedValue("/downloads/wazuh-ossec-logs-1.txt");
    const { result } = renderHook(() => useLogDownloader());

    await runDownload(result, "log data", "ossec");

    expect(result.current.saveState).toBe("success");
    expect(result.current.saveMsg).toBe("Saved to: /downloads/wazuh-ossec-logs-1.txt");
  });

  it("calls invoke with correct command, content and filename", async () => {
    mockInvoke.mockResolvedValue("/downloads/file.txt");
    const { result } = renderHook(() => useLogDownloader());

    await runDownload(result, "my logs", "update");

    expect(mockInvoke).toHaveBeenCalledWith("download_logs", expect.objectContaining({
      content: "my logs",
      filename: expect.stringMatching(/^wazuh-update-logs-\d+\.txt$/),
    }));
  });

  it("transitions to error and shows message when invoke rejects", async () => {
    mockInvoke.mockRejectedValue(new Error("Disk full"));
    const { result } = renderHook(() => useLogDownloader());

    await runDownload(result);

    expect(result.current.saveState).toBe("error");
    expect(result.current.saveMsg).toBe("Disk full");
  });

  it("resets back to idle after 4 seconds", async () => {
    mockInvoke.mockResolvedValue("/downloads/file.txt");
    const { result } = renderHook(() => useLogDownloader());

    await runDownload(result);
    expect(result.current.saveState).toBe("success");

    act(() => { vi.advanceTimersByTime(4000); });

    expect(result.current.saveState).toBe("idle");
    expect(result.current.saveMsg).toBeNull();
  });
});
