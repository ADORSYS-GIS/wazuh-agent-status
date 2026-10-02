import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export type SaveState = "idle" | "loading" | "success" | "error";

export function useLogDownloader() {
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const [saveMsg, setSaveMsg] = useState<string | null>(null);
  const downloadTimerRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (downloadTimerRef.current) window.clearTimeout(downloadTimerRef.current);
    };
  }, []);

  const downloadLogs = async (content: string, prefix: string) => {
    setSaveState("loading");
    setSaveMsg(null);
    try {
      const timestamp = Math.floor(Date.now() / 1000);
      const filename = `wazuh-${prefix}-logs-${timestamp}.txt`;
      const path = await invoke<string>("download_logs", { content, filename });
      setSaveState("success");
      setSaveMsg(`Saved to: ${path}`);
    } catch (e) {
      setSaveState("error");
      setSaveMsg(String(e instanceof Error ? e.message : e));
    } finally {
      if (downloadTimerRef.current) window.clearTimeout(downloadTimerRef.current);
      downloadTimerRef.current = window.setTimeout(() => {
        setSaveState("idle");
        setSaveMsg(null);
      }, 4000);
    }
  };

  return { saveState, saveMsg, downloadLogs };
}
