use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiModel {
    pub id: String,
    #[serde(default)]
    pub object: String,
    #[serde(default)]
    pub created: u64,
    #[serde(default)]
    pub owned_by: String,
}

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

pub struct AiClient {
    config: AiProviderConfig,
    http: reqwest::Client,
}

impl AiClient {
    pub fn new(config: AiProviderConfig) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(5)))
            .build()
            .map_err(|e| format!("Failed to build AI HTTP client: {e}"))?;

        Ok(Self { config, http })
    }

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
            .ok_or_else(|| "AI returned an empty response".to_string())
    }

    pub async fn ping(&self) -> Result<String, String> {
        if self.config.api_key.is_empty() {
            return Err("API key is required".to_string());
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
        if status.is_success() {
            Ok("Connected".to_string())
        } else if status.as_u16() == 404 {
            Ok("Connected (models endpoint not exposed)".to_string())
        } else if status.as_u16() == 401 || status.as_u16() == 403 {
            Err(format!("Authentication failed (HTTP {status})"))
        } else {
            Err(format!("Provider returned HTTP {status}"))
        }
    }

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
            return Ok(Vec::new());
        }

        let data: ModelsResponse = serde_json::from_str(&raw)
            .map_err(|e| format!("Failed to parse models response: {e}"))?;

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
