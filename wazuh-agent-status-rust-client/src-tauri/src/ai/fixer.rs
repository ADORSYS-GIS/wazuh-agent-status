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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_fix_prompt() {
        let input = FailedCheckInput {
            title: "Ensure password expiration is 90 days".to_string(),
            remediation: "Set PASS_MAX_DAYS 90 in /etc/login.defs".to_string(),
            os: "ubuntu".to_string(),
            mandatory: true,
            category: "Identity and Access".to_string(),
        };

        let prompt = build_fix_prompt(&input);
        assert!(prompt.contains("Ensure password expiration is 90 days"));
        assert!(prompt.contains("Mandatory**: Yes"));
        assert!(prompt.contains("ubuntu"));
        assert!(prompt.contains("/etc/login.defs"));
    }

    #[test]
    fn test_failed_check_input_and_result_serde() {
        let input = FailedCheckInput {
            title: "Test check".to_string(),
            remediation: "Fix it".to_string(),
            os: "linux".to_string(),
            mandatory: false,
            category: "General".to_string(),
        };

        let json = serde_json::to_string(&input).unwrap();
        let parsed: FailedCheckInput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, "Test check");

        let res = AiFixResult {
            markdown: "## Fix".to_string(),
            success: true,
            error: None,
        };
        let res_json = serde_json::to_string(&res).unwrap();
        let res_parsed: AiFixResult = serde_json::from_str(&res_json).unwrap();
        assert!(res_parsed.success);
        assert_eq!(res_parsed.markdown, "## Fix");
    }

    #[tokio::test]
    async fn test_generate_fixes_batch_empty() {
        let batch = generate_fixes_batch(vec![]).await;
        assert!(batch.is_empty());
    }

    #[tokio::test]
    async fn test_generate_fix_execution() {
        let input = FailedCheckInput {
            title: "Check SSH".to_string(),
            remediation: "Disable root login".to_string(),
            os: "linux".to_string(),
            mandatory: true,
            category: "SSH".to_string(),
        };

        let result = generate_fix(input.clone()).await;
        // Handles missing or invalid config gracefully without panic
        assert!(!result.success || !result.markdown.is_empty());

        let batch = generate_fixes_batch(vec![input]).await;
        assert_eq!(batch.len(), 1);
    }
}
