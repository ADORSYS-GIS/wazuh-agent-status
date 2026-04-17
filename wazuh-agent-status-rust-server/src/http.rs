//! Shared networking utilities for the Wazuh Agent Status server.

use std::time::Duration;
use reqwest::Client;

/// Generic HTTP GET request that returns raw bytes.
///
/// This provides a single point of configuration for timeouts and error
/// handling for all remote calls (manifest fetching and script downloads).
pub async fn fetch_bytes(url: &str, timeout: Duration) -> anyhow::Result<Vec<u8>> {
    let client = Client::builder()
        .timeout(timeout)
        .build()?;

    let resp = client.get(url)
        .send()
        .await?
        .error_for_status()?;

    let bytes = resp.bytes().await?.to_vec();
    Ok(bytes)
}
