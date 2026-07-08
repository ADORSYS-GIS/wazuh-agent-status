//! Provider-agnostic AI client abstraction.
//!
//! Uses the OpenAI Chat Completions API format, which is supported by most
//! major providers (OpenAI, Anthropic via proxy, Ollama, DeepSeek, Groq, etc.).
//!
//! # Examples
//!
//! ```ignore
//! let config = AiProviderConfig {
//!     base_url: "https://api.openai.com/v1".into(),
//!     api_key: "sk-...".into(),
//!     model: "gpt-4o".into(),
//!     ..Default::default()
//! };
//! let client = AiClient::new(config)?;
//! let answer = client.ask("Why is the sky blue?").await?;
//! ```

use serde::{Deserialize, Serialize};
use std::time::Duration;

// ── Public Configuration ──────────────────────────────────────────────────────

/// Describes which AI provider and model to use.
///
/// `base_url` and `model` are stored alongside the API key so the frontend
/// can display them, while the actual `api_key` is **never** sent back to the
/// renderer — it is only held in-memory during a Tauri command invocation
/// after being fetched from the OS keychain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    /// Base URL of the OpenAI-compatible API (e.g. `"https://api.openai.com/v1"`).
    pub base_url: String,
    /// API key / bearer token.
    #[serde(skip_serializing)]
    pub api_key: String,
    /// Model identifier (e.g. `"gpt-4o"`, `"claude-sonnet-4-20250514"`).
    pub model: String,
    /// Optional custom system prompt override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Request timeout in seconds (default 30).
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    30
}

impl Default for AiProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".into(),
            api_key: String::new(),
            model: "gpt-4o".into(),
            system_prompt: None,
            timeout_secs: 30,
        }
    }
}

/// Public subset of [`AiProviderConfig`] that is safe to send to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct AiProviderStatus {
    pub base_url: String,
    pub model: String,
    pub configured: bool,
}

impl From<&AiProviderConfig> for AiProviderStatus {
    fn from(c: &AiProviderConfig) -> Self {
        Self {
            base_url: c.base_url.clone(),
            model: c.model.clone(),
            configured: !c.api_key.is_empty(),
        }
    }
}

/// A model listed by the provider's `/v1/models` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModel {
    /// Model ID (e.g. `"gpt-4o"`, `"deepseek-chat"`).
    pub id: String,
    /// Object type (typically `"model"`).
    #[serde(default)]
    pub object: String,
    /// Unix timestamp of model creation.
    #[serde(default)]
    pub created: u64,
    /// Organization that owns the model.
    #[serde(default)]
    pub owned_by: String,
}

// ── Request / Response types (OpenAI-compatible) ──────────────────────────────

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    #[serde(default)]
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

/// Response from the `/v1/models` endpoint.
#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelsDataItem>,
    #[serde(default)]
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct ModelsDataItem {
    id: String,
    #[serde(default)]
    object: String,
    #[serde(default)]
    created: u64,
    #[serde(default)]
    owned_by: String,
}

// ── AI Client ─────────────────────────────────────────────────────────────────

/// A lightweight, provider-agnostic AI client.
///
/// All communication goes through the OpenAI-compatible `/v1/chat/completions`
/// endpoint, so **any** provider that mirrors this API can be used by simply
/// pointing [`AiProviderConfig::base_url`] at it.
pub struct AiClient {
    config: AiProviderConfig,
    http: reqwest::Client,
}

impl AiClient {
    /// Build a new client from the given configuration.
    pub fn new(config: AiProviderConfig) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(5)))
            .build()
            .map_err(|e| format!("Failed to build AI HTTP client: {e}"))?;

        Ok(Self { config, http })
    }

    /// Send a chat prompt and return the model's text response.
    pub async fn chat(&self, prompt: &str) -> Result<String, String> {
        let endpoint = self.config.base_url.trim_end_matches('/').to_string() + "/chat/completions";

        let body = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: self
                        .config
                        .system_prompt
                        .clone()
                        .unwrap_or_else(|| DEFAULT_SCA_FIX_PROMPT.to_string()),
                },
                Message {
                    role: "user".into(),
                    content: prompt.to_string(),
                },
            ],
            temperature: Some(0.3),
            max_tokens: Some(2048),
        };

        let resp = self
            .http
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("AI request failed: {e}"))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .unwrap_or_else(|_| "<no body>".to_string());

        if !status.is_success() {
            return Err(format!("AI provider returned HTTP {status}: {raw}"));
        }

        let data: ChatCompletionResponse = serde_json::from_str(&raw)
            .map_err(|e| format!("Failed to parse AI response: {e} — body: {raw}"))?;

        if let Some(err) = data.error {
            return Err(format!("AI API error: {}", err.message));
        }

        data.choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| "AI returned an empty response — no content in choice".to_string())
    }

    /// Quick connectivity check — hits `GET /v1/models` to verify the provider is reachable.
    ///
    /// Unlike [`chat`](Self::chat), this does **not** require a valid model,
    /// so it works even before the user has selected one. If the provider
    /// does not expose the models endpoint, a 404 or similar is accepted
    /// as long as the host is reachable.
    pub async fn ping(&self) -> Result<String, String> {
        if self.config.api_key.is_empty() {
            return Err("API key is required — configure a valid key before testing".to_string());
        }

        let endpoint = self.config.base_url.trim_end_matches('/').to_string() + "/models";

        let resp = self
            .http
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|e| format!("Provider unreachable: {e}"))?;

        let status = resp.status();
        // Accept any 2xx as success; also accept 404/401/403 (provider reachable
        // but models endpoint may not be exposed — that's fine).
        if status.is_success() {
            Ok("Connected — provider is reachable".to_string())
        } else if status.as_u16() == 404 {
            // Models endpoint not found — provider is still reachable
            Ok("Connected — provider reachable (models endpoint not exposed)".to_string())
        } else if status.as_u16() == 401 || status.as_u16() == 403 {
            Err(format!(
                "Authentication failed — check your API key (HTTP {status})"
            ))
        } else {
            Err(format!(
                "Provider returned HTTP {status} — check the URL and try again"
            ))
        }
    }

    /// Fetch available models from the provider's `/v1/models` endpoint.
    ///
    /// Returns a list of model IDs (e.g. `"gpt-4o"`, `"deepseek-chat"`).
    /// Some providers (e.g. Ollama) may return an empty list or an error
    /// if the endpoint is not exposed.
    pub async fn list_models(&self) -> Result<Vec<AiModel>, String> {
        let endpoint = self.config.base_url.trim_end_matches('/').to_string() + "/models";

        let resp = self
            .http
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch models: {e}"))?;

        let status = resp.status();
        let raw = resp
            .text()
            .await
            .unwrap_or_else(|_| "<no body>".to_string());

        if !status.is_success() {
            // Non-fatal: many providers don't expose the models endpoint
            return Ok(Vec::new());
        }

        let data: ModelsResponse = serde_json::from_str(&raw)
            .map_err(|e| format!("Failed to parse models response: {e} — body: {raw}"))?;

        if let Some(err) = data.error {
            return Err(format!("Models API error: {}", err.message));
        }

        Ok(data
            .data
            .into_iter()
            .map(|m| AiModel {
                id: m.id,
                object: m.object,
                created: m.created,
                owned_by: m.owned_by,
            })
            .collect())
    }
}

// ── Default system prompt ─────────────────────────────────────────────────────

const DEFAULT_SCA_FIX_PROMPT: &str = r#"You are a security configuration assistant for Wazuh agents.
Your role is to help system administrators fix failed Security Configuration Assessment (SCA) checks.

Given a failed SCA check, provide:
1. An explicit assessment of the risk level of applying this fix.
2. A description of the impact/side effects (such as service restarts, disconnection, or potential downtime).
3. A brief explanation of why this check is important.
4. The exact CLI commands needed to fix the issue.
5. A command to verify the fix was applied correctly.

CRITICAL COMMAND RULES (strictly enforced):
- NEVER suggest interactive editors like nano, vim, vi, emacs, gedit, or any command requiring keyboard input.
- ALL commands must be fully non-interactive and scriptable — they will be executed automatically by a GUI application.
- To modify file contents, use: sed -i, tee, printf, echo with redirection (>>/>), or awk.
- To append/create config file lines, prefer: echo "setting=value" | sudo tee -a /path/to/file
- To set a value in a config file, prefer: sudo sed -i 's/^#*PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config
- Each code block must be a single self-contained command or a short pipeline — no multi-step heredocs unless absolutely necessary.

Format your response EXACTLY as:
## Risk Level
[Specify exactly one of: Low, Medium, High]

## Impact
[Describe potential side-effects, service restarts, or system downtime]

## Explanation
[Brief explanation of why this check is important]

## Fix Steps
1. [Step 1]
2. [Step 2]
...

Include the exact commands in standard markdown fenced code blocks using bash/sh syntax on Linux or powershell/cmd on Windows, for example:
```bash
sudo sed -i 's/^#*PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config
```

## Verification
[Single non-interactive command to verify the fix was applied]

Be concise, precise, and provide copy-paste-ready non-interactive commands only."#;
