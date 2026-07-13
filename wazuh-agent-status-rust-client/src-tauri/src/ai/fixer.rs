use crate::ai::client::AiClient;
use crate::ai::keychain;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedCheckInput {
    pub title: String,
    pub remediation: String,
    pub os: String,
    pub mandatory: bool,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFixResult {
    pub markdown: String,
    pub success: bool,
    pub error: Option<String>,
}

fn build_fix_prompt(input: &FailedCheckInput) -> String {
    format!(
        r#"I need to fix a failed Wazuh SCA security check on my system.

## Context
- **Check Title**: {title}
- **Category**: {category}
- **Mandatory**: {mandatory}
- **Operating System**: {os}

## Remediation from Policy
{remediation}

Please provide the exact steps to fix this issue, including all CLI commands and configuration changes needed. Focus on practical, copy-paste-ready commands specific to {os}."#,
        title = input.title,
        category = input.category,
        mandatory = if input.mandatory { "Yes" } else { "No" },
        os = input.os,
        remediation = input.remediation,
    )
}

pub async fn generate_fix(input: FailedCheckInput) -> AiFixResult {
    let config = match keychain::get_config() {
        Ok(c) => c,
        Err(e) => {
            return AiFixResult {
                markdown: String::new(),
                success: false,
                error: Some(format!("AI not configured: {e}")),
            };
        }
    };

    let client = match AiClient::new(config) {
        Ok(c) => c,
        Err(e) => {
            return AiFixResult {
                markdown: String::new(),
                success: false,
                error: Some(e),
            };
        }
    };

    let prompt = build_fix_prompt(&input);

    match client.chat(&prompt).await {
        Ok(markdown) => AiFixResult {
            markdown,
            success: true,
            error: None,
        },
        Err(e) => AiFixResult {
            markdown: String::new(),
            success: false,
            error: Some(e),
        },
    }
}

pub async fn generate_fixes_batch(inputs: Vec<FailedCheckInput>) -> Vec<AiFixResult> {
    const INTER_REQUEST_DELAY_MS: u64 = 300;

    let total = inputs.len();
    let mut results = Vec::with_capacity(total);

    for (i, input) in inputs.into_iter().enumerate() {
        results.push(generate_fix(input).await);
        if i + 1 < total {
            tokio::time::sleep(tokio::time::Duration::from_millis(INTER_REQUEST_DELAY_MS)).await;
        }
    }

    results
}
