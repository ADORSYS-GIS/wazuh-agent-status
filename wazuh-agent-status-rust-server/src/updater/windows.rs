//! Windows update implementation.
//!
//! Regular update uses a 3-tier escalation strategy matching the Go server:
//!   1. PowerShell Scheduled Task (runs with SYSTEM/admin privileges)
//!   2. WMI Win32_Process in the active user session (fallback)
//!   3. `Start-Process -Verb RunAs` (last resort)
//!
//! Prerelease update runs the `.bat` file with the `-Prerelease` flag.

use std::sync::Arc;

use tokio::sync::mpsc;

use crate::config::AgentPaths;
use crate::updater::send_progress;
use crate::version_utils::fetch_version_info;

pub async fn run_update(
    paths: &Arc<AgentPaths>,
    version_url: &str,
    prerelease: bool,
    tx: mpsc::Sender<String>,
) {
    send_progress(&tx, "Starting...").await;

    let result = if prerelease {
        run_prerelease_update(paths, version_url, &tx).await
    } else {
        run_stable_update(paths, &tx).await
    };

    match result {
        Ok(()) => send_progress(&tx, "Complete").await,
        Err(e) => send_progress(&tx, &format!("Error: {e}")).await,
    }
}

// ── Stable update — 3-tier strategy ──────────────────────────────────────────

async fn run_stable_update(
    paths: &Arc<AgentPaths>,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    send_progress(tx, "Updating to stable").await;

    let script_path = &paths.update_script;
    if !script_path.exists() {
        anyhow::bail!("Wazuh update script not found at {}. Ensure the agent is installed.", script_path.display());
    }
    let script = script_path.to_string_lossy().into_owned();

    if run_scheduled_task(&script).await.is_ok() {
        return Ok(());
    }
    send_progress(tx, "Scheduled task failed; trying WMI...").await;

    if run_via_wmi(&script).await.is_ok() {
        return Ok(());
    }
    send_progress(tx, "WMI failed; trying direct execution...").await;

    run_direct(&script).await
}

async fn run_scheduled_task(script: &str) -> anyhow::Result<()> {
    const TASK_NAME: &str = "WazuhAgentUpdate";

    let ps_script = format!(
        r#"
        $taskName = "{TASK_NAME}"
        $existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
        if ($existingTask) {{ Unregister-ScheduledTask -TaskName $taskName -Confirm:$false }}
        $action   = New-ScheduledTaskAction -Execute "{script}" -Argument "-Update"
        $trigger  = New-ScheduledTaskTrigger -Once -At (Get-Date).AddSeconds(2)
        $principal = New-ScheduledTaskPrincipal -GroupId "S-1-5-32-544" -RunLevel Highest
        $settings  = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -StartWhenAvailable
        Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force
        Start-ScheduledTask -TaskName $taskName
        Start-Sleep -Seconds 2
        Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
        "#,
    );

    run_powershell(&ps_script).await
}

async fn run_via_wmi(script: &str) -> anyhow::Result<()> {
    let ps_script = format!(
        r#"
        $sessions = Get-CimInstance -ClassName Win32_ComputerSystem | Select-Object -ExpandProperty UserName
        if ($sessions) {{
            $sessionId = (Get-Process -IncludeUserName | Where-Object {{ $_.UserName -eq $sessions }} | Select-Object -First 1).SessionId
            if ($sessionId) {{
                $startInfo = ([wmiclass]"\\localhost\root\cimv2:Win32_ProcessStartup").CreateInstance()
                $startInfo.ShowWindow = 1
                $result = ([wmiclass]"\\localhost\root\cimv2:Win32_Process").Create("{script} -Update", $null, $startInfo)
                if ($result.ReturnValue -ne 0) {{ throw "WMI Create failed: $($result.ReturnValue)" }}
            }} else {{ throw "No session ID found" }}
        }} else {{ throw "No active user session" }}
        "#,
    );

    run_powershell(&ps_script).await
}

async fn run_direct(script: &str) -> anyhow::Result<()> {
    let ps_script = format!(
        r#"Start-Process -FilePath "{script}" -ArgumentList "-Update" -Verb RunAs -WindowStyle Normal"#
    );
    run_powershell(&ps_script).await
}

// ── Prerelease update ─────────────────────────────────────────────────────────

async fn run_prerelease_update(
    paths: &Arc<AgentPaths>,
    version_url: &str,
    tx: &mpsc::Sender<String>,
) -> anyhow::Result<()> {
    send_progress(tx, "Fetching prerelease version info...").await;

    let info = fetch_version_info(version_url)
        .await
        .ok_or_else(|| anyhow::anyhow!("Failed to fetch version manifest"))?;

    if info.framework.prerelease_version.is_empty() {
        anyhow::bail!("No prerelease version available in manifest");
    }

    send_progress(tx, "Updating to prerelease").await;

    let script = paths.update_script.to_string_lossy().into_owned();
    let ps_script = format!(r#"& "{script}" -Prerelease"#);
    run_powershell(&ps_script).await
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn run_powershell(command: &str) -> anyhow::Result<()> {
    let status = tokio::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", command])
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("PowerShell command exited with {status}");
    }
    Ok(())
}
