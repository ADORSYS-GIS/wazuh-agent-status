import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "../types/app";
import type { AiProviderConfig, AiProviderStatus, AiModel } from "../types/ai";

const stripSlash = (s: string) => {
  let end = s.length;
  while (end > 0 && s[end - 1] === "/") {
    end--;
  }
  return s.slice(0, end);
};

interface SettingsViewProps {
  config: AppConfig;
}

export function SettingsView({ config }: Readonly<SettingsViewProps>) {
  const { managed_by, company } = config.brand;
  const managedByName = managed_by ?? company;
  const showCustomer = managed_by !== undefined && managed_by !== company;

  // ── AI Provider State ─────────────────────────────────────────────────
  const [aiStatus, setAiStatus] = useState<AiProviderStatus | null>(null);

  // Form fields
  const [baseUrl, setBaseUrl] = useState("https://api.ai.camer.digital/v1");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("deepseek-v4-flash");

  // Model listing
  const [models, setModels] = useState<AiModel[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);
  const [modelFetchError, setModelFetchError] = useState<string | null>(null);
  const [useModelSelect, setUseModelSelect] = useState(true);

  // UI state
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [testError, setTestError] = useState<string | null>(null);
  const [testOk, setTestOk] = useState(false);
  const [saveMsg, setSaveMsg] = useState<{ text: string; ok: boolean } | null>(null);
  const [showForm, setShowForm] = useState(false);

  // ── Auto-dismiss status messages after 5s ────────────────────────────
  useEffect(() => {
    if (!saveMsg) return;
    const timer = setTimeout(() => setSaveMsg(null), 5000);
    return () => clearTimeout(timer);
  }, [saveMsg]);

  useEffect(() => {
    if (!testError) return;
    const timer = setTimeout(() => setTestError(null), 5000);
    return () => clearTimeout(timer);
  }, [testError]);

  useEffect(() => {
    if (!testOk) return;
    const timer = setTimeout(() => setTestOk(false), 5000);
    return () => clearTimeout(timer);
  }, [testOk]);

  // ── Fetch saved config on mount (never fails — returns { configured: false } if unset) ─
  const fetchAiStatus = useCallback(async () => {
    const status = await invoke<AiProviderStatus>("get_ai_status");
    setAiStatus(status.configured ? status : null);
    if (status.configured) {
      setBaseUrl(status.base_url);
      setModel(status.model);
    }
  }, []);

  useEffect(() => { fetchAiStatus(); }, [fetchAiStatus]);

  // ── Fetch models ──────────────────────────────────────────────────────
  const fetchModels = useCallback(async (url: string, key: string) => {
    setLoadingModels(true);
    setModelFetchError(null);
    try {
      const cfg: AiProviderConfig = { base_url: stripSlash(url), api_key: key, model: "" };
      const result = await invoke<AiModel[]>("list_ai_models", { config: cfg });
      setModels(result);
      if (result.length === 0) setModelFetchError("No models returned by provider — type a model name manually");
    } catch {
      setModels([]);
    } finally {
      setLoadingModels(false);
    }
  }, []);

  // ── Test connection ───────────────────────────────────────────────────
  const handleTest = useCallback(async () => {
    const cleanUrl = stripSlash(baseUrl);
    if (!cleanUrl) { setTestError("Enter a base URL"); return; }
    setTesting(true);
    setTestError(null);
    setTestOk(false);
    setModels([]);
    try {
      const cfg: AiProviderConfig = { base_url: cleanUrl, api_key: apiKey, model: model.trim() || "deepseek-v4-flash" };
      await invoke<string>("test_ai_connection", { config: cfg });
      setTestOk(true);
      fetchModels(cleanUrl, apiKey);
    } catch (e) {
      setTestError(String(e));
    } finally {
      setTesting(false);
    }
  }, [baseUrl, apiKey, model, fetchModels]);

  // ── Save ──────────────────────────────────────────────────────────────
  const handleSave = useCallback(async () => {
    const cleanUrl = stripSlash(baseUrl);
    if (!cleanUrl) { setSaveMsg({ text: "Base URL is required", ok: false }); return; }
    if (!model.trim()) { setSaveMsg({ text: "Select or type a model name", ok: false }); return; }
    if (!apiKey.trim()) { setSaveMsg({ text: "API key is required to save the configuration", ok: false }); return; }
    setSaving(true);
    setSaveMsg(null);
    try {
      const cfg: AiProviderConfig = { base_url: cleanUrl, api_key: apiKey, model: model.trim() };
      await invoke("save_ai_config", { config: cfg });
      setAiStatus({ base_url: cleanUrl, model: model.trim(), configured: true });
      setShowForm(false);
    } catch (e) {
      setSaveMsg({ text: `Failed to save: ${e}`, ok: false });
    } finally {
      setSaving(false);
    }
  }, [baseUrl, apiKey, model]);

  // ── Disconnect ────────────────────────────────────────────────────────
  const handleDisconnect = useCallback(async () => {
    try {
      await invoke("clear_ai_config");
      setAiStatus(null);
      setApiKey("");
      setModels([]);
      setShowForm(true);
      setBaseUrl("https://api.ai.camer.digital/v1");
      setModel("deepseek-v4-flash");
      setSaveMsg({ text: "Disconnected", ok: true });
    } catch (e) {
      setSaveMsg({ text: `Failed: ${e}`, ok: false });
    }
  }, []);

  // ── Render ────────────────────────────────────────────────────────
  return (
    <div className="view-container">
      <div className="subtitle">System Information</div>
      <h2 className="header title">App Settings</h2>

      <div className="card">
        <div className="card-info">
          <div className="card-label">Managed By</div>
          <div className="card-value">{managedByName}</div>
        </div>
      </div>

      {showCustomer && (
        <div className="card">
          <div className="card-info">
            <div className="card-label">Customer</div>
            <div className="card-value">{company}</div>
          </div>
        </div>
      )}
      
      <div className="card">
        <div className="card-info">
          <div className="card-label">Environment</div>
          <div className="card-value">Production</div>
        </div>
      </div>

      {/* ── AI Provider ─────────────────────────────────────────── */}
      <div className="section-title section-title--spaced">AI Provider</div>

      {aiStatus && !showForm ? (
        /* ── Connected State ──────────────────────────────────────── */
        <div className="settings-ai-connected">
          <div className="settings-ai-connected-row">
            <div className="settings-ai-connected-icon">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M12 2l2.4 7.2H22l-6 4.8 2.4 7.2L12 16l-6 4.8L8.4 14l-6-4.8h7.6z" />
              </svg>
            </div>
            <div className="settings-ai-connected-body">
              <div className="settings-ai-connected-name">
                {aiStatus.base_url
                  .replace(/^https?:\/\//, "")}
              </div>
              <div className="settings-ai-connected-model">{aiStatus.model}</div>
            </div>
            <div className="settings-ai-connected-badge">
              <span className="settings-ai-dot" />
              <span>Connected</span>
            </div>
          </div>
          <div className="settings-ai-connected-actions">
            <button
              className="settings-ai-btn settings-ai-btn-secondary"
              onClick={() => {
                setShowForm(true);
                setBaseUrl(aiStatus.base_url);
                setModel(aiStatus.model);
              }}
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                <path d="M12 20h9" />
                <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
              </svg>
              Change
            </button>
            <button className="settings-ai-btn settings-ai-btn-danger" onClick={handleDisconnect}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
              Disconnect
            </button>
          </div>
        </div>
      ) : (
        /* ── Setup Form ──────────────────────────────────────────── */
        <div className="settings-ai-setup">
          <div className="settings-ai-form">
            {/* URL + Key side by side */}
            <div className="settings-ai-form-row">
              <div className="settings-ai-field">
                <label className="settings-ai-label" htmlFor="ai-base-url">API Base URL</label>
                <input
                  id="ai-base-url"
                  className="settings-ai-input"
                  type="text"
                  placeholder="https://api.ai.camer.digital/v1"
                  value={baseUrl}
                  onChange={(e) => {
                    setBaseUrl(e.target.value);
                    setTestOk(false);
                    setTestError(null);
                    setModels([]);
                  }}
                  spellCheck={false}
                  autoComplete="off"
                />
              </div>
              <div className="settings-ai-field">
                <label className="settings-ai-label" htmlFor="ai-api-key">API Key</label>
                <input
                  id="ai-api-key"
                  className="settings-ai-input"
                  type="password"
                  placeholder="sk-..."
                  value={apiKey}
                  onChange={(e) => {
                    setApiKey(e.target.value);
                    setTestOk(false);
                    setTestError(null);
                    setModels([]);
                  }}
                  spellCheck={false}
                  autoComplete="off"
                />
              </div>
            </div>

            {/* Model — always editable, with dropdown overlay when models are loaded */}
            <div className="settings-ai-field">
              <div className="settings-ai-label-row">
                <label className="settings-ai-label" htmlFor="ai-model">Model</label>
                <div className="settings-ai-model-actions">
                  {models.length > 0 && (
                    <span className="settings-ai-skip-hint">{models.length} available</span>
                  )}
                  <button
                    className="settings-ai-fetch-btn"
                    disabled={loadingModels || !baseUrl.trim()}
                    onClick={() => fetchModels(stripSlash(baseUrl), apiKey)}
                  >
                    {loadingModels ? (
                      <><span className="settings-ai-spinner" /> Loading</>
                    ) : (
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                        <polyline points="23 4 23 10 17 10" />
                        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
                      </svg>
                    )}
                    Fetch models
                  </button>
                </div>
              </div>
              {models.length > 0 && useModelSelect ? (
                <>
                  <select
                    id="ai-model"
                    className="settings-ai-select"
                    value={models.some((m) => m.id === model) ? model : "__custom__"}
                    onChange={(e) => {
                      if (e.target.value === "__custom__") {
                        setUseModelSelect(false);
                      } else {
                        setModel(e.target.value);
                      }
                    }}
                  >
                    {models.map((m) => (
                      <option key={m.id} value={m.id}>
                        {m.id}
                      </option>
                    ))}
                    <option value="__custom__">── Type custom ──</option>
                  </select>
                  <span className="settings-ai-hint">
                    <button
                      className="settings-ai-link-btn"
                      onClick={() => setUseModelSelect(false)}
                    >
                      Or type a custom model
                    </button>
                  </span>
                </>
              ) : (
                <>
                  <input
                    id="ai-model"
                    className="settings-ai-input"
                    type="text"
                    placeholder="deepseek-v4-flash, gpt-4o..."
                    value={model}
                    onChange={(e) => setModel(e.target.value)}
                    spellCheck={false}
                  />
                  {models.length > 0 && !useModelSelect && (
                    <span className="settings-ai-hint">
                      <button
                        className="settings-ai-link-btn"
                        onClick={() => setUseModelSelect(true)}
                      >
                        Show {models.length} available models
                      </button>
                    </span>
                  )}
                </>
              )}
              {loadingModels && <span className="settings-ai-hint">Loading models...</span>}
              {modelFetchError && !loadingModels && <span className="settings-ai-hint">{modelFetchError}</span>}
              {!loadingModels && !modelFetchError && models.length === 0 && (
                <span className="settings-ai-hint">Type a model name or click "Fetch models" to load from provider</span>
              )}
            </div>

            {/* Status messages */}
            {testError && (
              <div className="settings-ai-msg error">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                  <circle cx="12" cy="12" r="10" />
                  <line x1="15" y1="9" x2="9" y2="15" />
                  <line x1="9" y1="9" x2="15" y2="15" />
                </svg>
                {testError}
              </div>
            )}
            {testOk && (
              <div className="settings-ai-msg success">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
                Connection successful
              </div>
            )}
            {saveMsg && (
              <div className={`settings-ai-msg ${saveMsg.ok ? "success" : "error"}`}>
                {saveMsg.ok ? (
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                ) : (
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
                    <circle cx="12" cy="12" r="10" />
                    <line x1="15" y1="9" x2="9" y2="15" />
                    <line x1="9" y1="9" x2="15" y2="15" />
                  </svg>
                )}
                {saveMsg.text}
              </div>
            )}

            {/* Action buttons */}
            <div className="settings-ai-actions">
              <button
                className="settings-ai-btn settings-ai-btn-outline"
                disabled={testing || saving}
                onClick={handleTest}
              >
                {testing ? (
                  <>
                    <span className="settings-ai-spinner" /> Testing...
                  </>
                ) : (
                  "Test Connection"
                )}
              </button>
              <button
                className="settings-ai-btn settings-ai-btn-primary"
                disabled={testing || saving}
                onClick={handleSave}
              >
                {saving ? (
                  <>
                    <span className="settings-ai-spinner" /> Saving...
                  </>
                ) : (
                  "Save Configuration"
                )}
              </button>
              {aiStatus && (
                <button
                  className="settings-ai-btn settings-ai-btn-ghost"
                  onClick={() => {
                    setShowForm(false);
                    fetchAiStatus();
                  }}
                >
                  Cancel
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
