// ── AI Provider Configuration ─────────────────────────────────────────────────

/** A model listed by the provider's /v1/models endpoint. */
export interface AiModel {
  id: string;
  object: string;
  created: number;
  owned_by: string;
}

export interface AiProviderConfig {
  /** Base URL of the OpenAI-compatible API (e.g. "https://api.openai.com/v1") */
  base_url: string;
  /** API key / bearer token */
  api_key: string;
  /** Model identifier (e.g. "gpt-4o", "claude-sonnet-4-20250514") */
  model: string;
  /** Optional custom system prompt override */
  system_prompt?: string | null;
  /** Request timeout in seconds */
  timeout_secs?: number;
}

/** Safe-to-display subset of the AI provider status (no API key). */
export interface AiProviderStatus {
  base_url: string;
  model: string;
  configured: boolean;
}

// ── SCA Fix ───────────────────────────────────────────────────────────────────

/** A failed SCA check the user wants AI help fixing. */
export interface FailedCheckInput {
  /** Title of the failed check */
  title: string;
  /** Pre-written remediation text from the compliance profile */
  remediation: string;
  /** Target OS (e.g. "Ubuntu 22.04") */
  os: string;
  /** Whether this is a mandatory check */
  mandatory: boolean;
  /** The compliance category this check belongs to */
  category: string;
}

/** A structured fix returned by the AI. */
export interface AiFixResult {
  /** Raw markdown from the AI model */
  markdown: string;
  /** Whether the AI call succeeded */
  success: boolean;
  /** Error message if success is false */
  error: string | null;
}

// ── Follow-up Chat ────────────────────────────────────────────────────────────

/** A single message in the AI follow-up chat. */
export interface ChatMessage {
  /** Unique identifier for list keys */
  id: number;
  role: "user" | "assistant";
  content: string;
}
