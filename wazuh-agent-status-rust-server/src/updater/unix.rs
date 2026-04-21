//! Unix (Linux + macOS) update implementation.
//!
//! Regular update:  executes the `adorsys-update.sh` script already present
//!                  on the system (no download required; the script itself
//!                  handles privilege escalation via its own sudo calls).
//!
//! Prerelease update: fetches the version manifest, downloads the correct
//!                    setup script to a temporary file, and runs it.

use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::config::AgentPaths;
use crate::updater::send_progress;
use crate::version_utils::fetch_version_info;

/// URL template for the prerelease setup script.
const PRERELEASE_SCRIPT_URL: &str =
    "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v{VERSION}/scripts/setup-agent.sh";

pub async fn run_update(
    paths: &Arc<AgentPaths>,
    version_url: &str,
    prerelease: bool,
    tx: mpsc::Sender<String>,
) {
    send_progress(&tx, "Starting...").await;

    let result = if prerelease {
        run_prerelease_update(version_url, &tx).await
    } else {
        run_stable_update(paths, &tx).await
    };

    match result {
        Ok(()) => send_progress(&tx, "Complete").await,
        Err(e) => send_progress(&tx, &format!("Error: {e}")).await,
    }
}

// ── Stable update ─────────────────────────────────────────────────────────────

async fn run_stable_update(
    paths: &Arc<AgentPaths>,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    send_progress(tx, "Updating to stable").await;

    let script = &paths.update_script;
    if !script.exists() {
        anyhow::bail!("Update script not found at {}. Ensure the Wazuh agent is correctly installed.", script.display());
    }

    let mut child = Command::new(script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn update script {}: {e}", script.display()))?;

    stream_output(&mut child, tx).await;

    let status = child.wait().await?;
    if !status.success() {
        anyhow::bail!("Update script exited with {status}");
    }
    Ok(())
}

// ── Prerelease update ─────────────────────────────────────────────────────────

async fn run_prerelease_update(
    version_url: &str,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    send_progress(tx, "Fetching prerelease version info...").await;

    let info = fetch_version_info(version_url)
        .await
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch version manifest"))?;

    let prerelease_version = info.framework.prerelease_version;
    if prerelease_version.is_empty() {
        anyhow::bail!("No prerelease version available in manifest");
    }

    let script_url = PRERELEASE_SCRIPT_URL.replace("{VERSION}", &prerelease_version);
    send_progress(tx, &format!("Downloading setup script for v{prerelease_version}...")).await;

    // Write to a temp file
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join(format!("wazuh-prerelease-{prerelease_version}.sh"));

    download_file(&script_url, &tmp_path).await?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o750))?;
    }

    send_progress(tx, "Updating to prerelease").await;

    let mut child = Command::new(&tmp_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn prerelease script: {e}"))?;

    stream_output(&mut child, tx).await;

    let status = child.wait().await?;

    // Always remove temp file, even on failure
    let _ = std::fs::remove_file(&tmp_path);

    if !status.success() {
        anyhow::bail!("Prerelease script exited with {status}");
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Stream stdout lines from the child process to the progress channel.
async fn stream_output(child: &mut tokio::process::Child, tx: &mpsc::Sender<String>) {
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            send_progress(tx, &line).await;
        }
    }
}

/// Download `url` to `dest` using the shared HTTP utility.
async fn download_file(url: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    let bytes = crate::http::fetch_bytes(url, std::time::Duration::from_secs(60)).await?;
    tokio::fs::write(dest, &bytes).await?;
    Ok(())
}
