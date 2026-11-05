use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub base_url: String,
    #[serde(skip_serializing)]
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
2. A description of the impact/side effects (such as service restarts, connectivity loss, service disruption, configuration lockout, or potential data loss). Be specific about what could go wrong and include a brief hint on how to recover or revert the change if needed and recover command should be ignore if not deterministic and clearly understandable.
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
[Describe potential side-effects, including connectivity loss, service disruptions, or configuration lockout. Also include a short recovery hint — for example: "If you lose connectivity, revert by running: <undo command>". or contact the security team to fix the issue]

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
Provide a single non-interactive deterministic command that produces a clear pass/fail output. The command MUST:
- Return exit code 0 if the fix is applied correctly
- Return exit code non-zero or produce no output if the fix is not applied
- Rely on the exact setting or runtime state, not on text searching (avoid grep)
- Be something the user can run immediately to confirm the fix took effect

Be concise, precise, and provide copy-paste-ready non-interactive commands only."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_provider_config_default_and_status() {
        let def = AiProviderConfig::default();
        assert_eq!(def.base_url, "https://api.openai.com/v1");
        assert_eq!(def.model, "gpt-4o");
        assert_eq!(def.timeout_secs, 30);
        assert!(def.api_key.is_empty());

        let status_unconfigured = AiProviderStatus::from(&def);
        assert!(!status_unconfigured.configured);

        let mut configured = def.clone();
        configured.api_key = "sk-test-key".to_string();
        let status_configured = AiProviderStatus::from(&configured);
        assert!(status_configured.configured);
        assert_eq!(status_configured.model, "gpt-4o");
    }

    #[test]
    fn test_ai_client_validation() {
        let valid = AiProviderConfig {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "valid-key".to_string(),
            model: "gpt-4o".to_string(),
            ..AiProviderConfig::default()
        };
        let client = AiClient::new(valid.clone()).unwrap();
        assert_eq!(client.config.base_url, "https://api.openai.com/v1");
        assert_eq!(client.config.model, "gpt-4o");
        assert_eq!(client.config.api_key, "valid-key");
    }

    #[test]
    fn test_system_prompt_non_empty() {
        assert!(DEFAULT_SCA_FIX_PROMPT.contains("Risk Level"));
        assert!(DEFAULT_SCA_FIX_PROMPT.contains("Verification"));
    }

    #[test]
    fn test_chat_response_deserialization() {
        let json = "{\"choices\":[{\"message\":{\"content\":\"Risk Level Low\"}}]}";
        let resp: ChatCompletionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.choices.len(), 1);
        assert_eq!(
            resp.choices[0].message.content.as_deref(),
            Some("Risk Level Low")
        );
    }

    #[tokio::test]
    async fn test_ai_client_ping_missing_key() {
        let config = AiProviderConfig {
            api_key: String::new(),
            ..AiProviderConfig::default()
        };
        let client = AiClient::new(config).unwrap();
        let res = client.ping().await;
        assert_eq!(res.unwrap_err(), "API key is required");
    }

    #[tokio::test]
    async fn test_ai_client_network_failures() {
        let config = AiProviderConfig {
            base_url: "http://127.0.0.1:9/v1".to_string(),
            api_key: "test-key".to_string(),
            timeout_secs: 1,
            ..AiProviderConfig::default()
        };
        let client = AiClient::new(config).unwrap();
        assert!(client.ping().await.is_err());
        assert!(client.chat("test").await.is_err());
        assert!(client.list_models().await.is_err());
    }

    #[test]
    fn test_models_response_deserialization() {
        let json = r#"{"data":[{"id":"gpt-4o","object":"model","created":1600000000,"owned_by":"system"}]}"#;
        let resp: ModelsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].id, "gpt-4o");

        let err_json = r#"{"data":[],"error":{"message":"quota exceeded"}}"#;
        let err_resp: ModelsResponse = serde_json::from_str(err_json).unwrap();
        assert_eq!(err_resp.error.unwrap().message, "quota exceeded");
    }

    #[test]
    fn test_chat_response_with_error_and_empty() {
        let err_json = r#"{"choices":[],"error":{"message":"server error"}}"#;
        let resp: ChatCompletionResponse = serde_json::from_str(err_json).unwrap();
        assert_eq!(resp.error.unwrap().message, "server error");

        let empty_json = r#"{"choices":[]}"#;
        let empty_resp: ChatCompletionResponse = serde_json::from_str(empty_json).unwrap();
        assert!(empty_resp.choices.is_empty());
    }

    // ── Mock HTTP provider ───────────────────────────────────────────────────

    /// Minimal HTTP server. Routes are matched by the first path segment
    /// (e.g. `/ok/...`, `/err500/...`) so each scenario gets its own base URL.
    async fn spawn_mock_provider() -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((mut sock, _)) = listener.accept().await {
                tokio::spawn(async move {
                    // Read headers, then drain the body according to Content-Length.
                    let mut buf = Vec::new();
                    let mut chunk = [0u8; 4096];
                    let header_end = loop {
                        let n = sock.read(&mut chunk).await.unwrap_or(0);
                        if n == 0 {
                            return;
                        }
                        buf.extend_from_slice(&chunk[..n]);
                        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                            break pos + 4;
                        }
                    };
                    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
                    let content_len = head
                        .lines()
                        .find_map(|l| {
                            let l = l.to_ascii_lowercase();
                            l.strip_prefix("content-length:")
                                .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                        })
                        .unwrap_or(0);
                    while buf.len() - header_end < content_len {
                        let n = sock.read(&mut chunk).await.unwrap_or(0);
                        if n == 0 {
                            break;
                        }
                        buf.extend_from_slice(&chunk[..n]);
                    }

                    let path = head.split_whitespace().nth(1).unwrap_or("/").to_string();
                    let scenario = path.trim_start_matches('/').split('/').next().unwrap_or("");
                    let is_chat = path.ends_with("/chat/completions");
                    let (status, body): (&str, &str) = match (scenario, is_chat) {
                        ("ok", true) => (
                            "200 OK",
                            r#"{"choices":[{"message":{"content":"fixed it"}}]}"#,
                        ),
                        ("ok", false) => (
                            "200 OK",
                            r#"{"data":[{"id":"gpt-4o","object":"model","created":1,"owned_by":"system"}]}"#,
                        ),
                        ("apierr", true) => (
                            "200 OK",
                            r#"{"choices":[],"error":{"message":"quota exceeded"}}"#,
                        ),
                        ("apierr", false) => (
                            "200 OK",
                            r#"{"data":[],"error":{"message":"quota exceeded"}}"#,
                        ),
                        ("empty", _) => ("200 OK", r#"{"choices":[]}"#),
                        ("badjson", _) => ("200 OK", "not json"),
                        ("unauth", _) => ("401 Unauthorized", ""),
                        ("forbidden", _) => ("403 Forbidden", ""),
                        ("nf", _) => ("404 Not Found", ""),
                        _ => ("500 Internal Server Error", "boom"),
                    };
                    let resp = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = sock.write_all(resp.as_bytes()).await;
                    let _ = sock.shutdown().await;
                });
            }
        });
        format!("http://{addr}")
    }

    fn client_for(base: &str, scenario: &str) -> AiClient {
        AiClient::new(AiProviderConfig {
            base_url: format!("{base}/{scenario}/"),
            api_key: "test-key".to_string(),
            model: "gpt-4o".to_string(),
            system_prompt: Some("custom prompt".to_string()),
            timeout_secs: 5,
        })
        .unwrap()
    }

    #[tokio::test]
    async fn test_chat_response_branches() {
        let base = spawn_mock_provider().await;

        assert_eq!(
            client_for(&base, "ok").chat("hi").await.unwrap(),
            "fixed it"
        );

        let err = client_for(&base, "err500").chat("hi").await.unwrap_err();
        assert!(err.contains("HTTP 500"), "{err}");
        assert!(err.contains("boom"), "{err}");

        let err = client_for(&base, "apierr").chat("hi").await.unwrap_err();
        assert!(err.contains("AI API error: quota exceeded"), "{err}");

        let err = client_for(&base, "empty").chat("hi").await.unwrap_err();
        assert_eq!(err, "AI returned an empty response");

        let err = client_for(&base, "badjson").chat("hi").await.unwrap_err();
        assert!(err.contains("Failed to parse AI response"), "{err}");
    }

    #[tokio::test]
    async fn test_ping_status_branches() {
        let base = spawn_mock_provider().await;

        assert_eq!(client_for(&base, "ok").ping().await.unwrap(), "Connected");
        assert_eq!(
            client_for(&base, "nf").ping().await.unwrap(),
            "Connected (models endpoint not exposed)"
        );

        let err = client_for(&base, "unauth").ping().await.unwrap_err();
        assert!(err.contains("Authentication failed"), "{err}");
        let err = client_for(&base, "forbidden").ping().await.unwrap_err();
        assert!(err.contains("Authentication failed"), "{err}");

        let err = client_for(&base, "err500").ping().await.unwrap_err();
        assert!(err.contains("Provider returned HTTP 500"), "{err}");
    }

    #[tokio::test]
    async fn test_list_models_branches() {
        let base = spawn_mock_provider().await;

        let models = client_for(&base, "ok").list_models().await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "gpt-4o");
        assert_eq!(models[0].owned_by, "system");

        // Non-success status yields an empty list rather than an error
        assert!(
            client_for(&base, "err500")
                .list_models()
                .await
                .unwrap()
                .is_empty()
        );

        let err = client_for(&base, "apierr").list_models().await.unwrap_err();
        assert!(err.contains("Models API error: quota exceeded"), "{err}");

        let err = client_for(&base, "badjson")
            .list_models()
            .await
            .unwrap_err();
        assert!(err.contains("Failed to parse models response"), "{err}");
    }
}
