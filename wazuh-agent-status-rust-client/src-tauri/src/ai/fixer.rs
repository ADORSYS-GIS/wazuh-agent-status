//! SCA fix generation — the bridge between failed compliance checks and AI.
//!
//! Takes a description of a failed SCA check, constructs a well-crafted prompt
//! that includes the check title, remediation text, and OS context, then calls
//! the configured AI provider and returns a structured, actionable fix.

use crate::ai::client::AiClient;
use crate::ai::keychain;
use serde::{Deserialize, Serialize};

// ── Input ─────────────────────────────────────────────────────────────────────

/// A failed SCA check that the user wants AI help fixing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedCheckInput {
    /// Title of the failed check (e.g. "Ensure sshd PermitRootLogin is disabled").
    pub title: String,
    /// Pre-written remediation text from the compliance profile.
    pub remediation: String,
    /// Target OS (e.g. "Ubuntu 22.04", "Windows Server 2022", "macOS 14").
    pub os: String,
    /// Whether this is a mandatory check.
    pub mandatory: bool,
    /// The compliance category this check belongs to.
    pub category: String,
}

// ── Output ────────────────────────────────────────────────────────────────────

/// A structured fix returned by the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiFixResult {
    /// Raw markdown from the AI model.
    pub markdown: String,
    /// Whether the AI call succeeded.
    pub success: bool,
    /// Error message if `success` is `false`.
    pub error: Option<String>,
}

// ── Prompt construction ───────────────────────────────────────────────────────

/// Build a detailed prompt for the AI to fix a failed SCA check.
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

// ── Public API ────────────────────────────────────────────────────────────────

/// Generate an AI-powered fix for a single failed SCA check.
///
/// 1. Reads the provider config from the OS keychain.
/// 2. Constructs an [`AiClient`] and sends the prompt.
/// 3. Returns a structured [`AiFixResult`].
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

/// Generate an AI-powered fix for **all** failed checks in a batch.
///
/// This sends each check as a separate prompt so the AI can give focused,
/// check-specific answers rather than one long confusing response.
pub async fn generate_fixes_batch(inputs: Vec<FailedCheckInput>) -> Vec<AiFixResult> {
    let mut results = Vec::with_capacity(inputs.len());
    for input in inputs {
        results.push(generate_fix(input).await);
    }
    results
}
