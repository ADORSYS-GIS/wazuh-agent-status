#requires -version 5.1

# ---- Parameters ----
param(
    [switch]$Prerelease,
    [switch]$Update
)

# Set strict mode for error handling
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ---- Elevate ----
$IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $IsAdmin) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName  = (Get-Process -Id $PID).Path
    $arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$($MyInvocation.MyCommand.Path)`""

    if ($Prerelease) {
        $arguments += " -Prerelease"
    }
    if ($Update) {
        $arguments += " -Update"
    }

    $psi.Arguments = $arguments
    $psi.Verb      = "runas"
    try {
        [System.Diagnostics.Process]::Start($psi) | Out-Null
        exit
    } catch {
        Write-Host "Administrator approval is required. Exiting."
        exit 1
    }
}

# Configuration

$APP_VERSION = if ($env:APP_VERSION) { $env:APP_VERSION } else { "0.5.2" }
$REPO_REF = if ($env:WAZUH_AGENT_STATUS_REPO_REF) { $env:WAZUH_AGENT_STATUS_REPO_REF } else { "main" }
$REPO_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$REPO_REF"
$AGENT_REPO_REF = if ($env:WAZUH_AGENT_REPO_REF) { $env:WAZUH_AGENT_REPO_REF } else { "main" }
$TMP = Join-Path $env:TEMP "wazuh-agent-status-install"; if (-not (Test-Path $TMP)) { mkdir $TMP | Out-Null }

try {
    $global:ChecksumsPath = Join-Path $TMP "checksums.sha256"; $U = Join-Path $TMP "utils.ps1"
    Invoke-WebRequest "$REPO_URL/checksums.sha256" -OutFile $global:ChecksumsPath; Invoke-WebRequest "$REPO_URL/scripts/shared/utils.ps1" -OutFile $U
    if ((Get-FileHash $U -Alg SHA256).Hash.ToLower() -ne (Select-String -Path $global:ChecksumsPath -Pattern "scripts/shared/utils.ps1").Line.Split(" ")[0].ToLower()) { throw }
    . $U
} catch { Write-Error "Bootstrap failed"; exit 1 }

# Override utils.ps1 logging functions to use Append-Log (Write-Host) and prevent pipeline array pollution
function Log { param([string]$Level, [string]$Message, [string]$Color = "White") Append-Log "$Level $Message" }
function InfoMessage { param([string]$Message) Append-Log $Message "INFO" }
function WarnMessage { param([string]$Message) Append-Log $Message "WARN" }
function ErrorMessage { param([string]$Message) Append-Log $Message "ERROR" }
function SuccessMessage { param([string]$Message) Append-Log $Message "SUCCESS" }

EnsureWindows
EnsureAdmin

# Cleanup bootstrap files on exit
Register-EngineEvent -SourceIdentifier ([System.Guid]::NewGuid().ToString()) -Action {
    Remove-Item -Path $TMP -Recurse -Force -ErrorAction SilentlyContinue
} | Out-Null

# ---- Configuration Variables ----
$CURRENT_MANAGER = $null
$OssecConfPath = "C:\Program Files (x86)\ossec-agent\ossec.conf"
if (Test-Path $OssecConfPath) {
    $addressLine = Select-String -Path $OssecConfPath -Pattern "<address>(.*?)</address>" | Select-Object -First 1
    if ($addressLine -match "<address>(.*?)</address>") {
        $CURRENT_MANAGER = $matches[1]
    }
}
$WAZUH_MANAGER           = if ($env:WAZUH_MANAGER) { $env:WAZUH_MANAGER } elseif ($CURRENT_MANAGER) { $CURRENT_MANAGER } else { "wazuh.example.com" }
$OSSEC_PATH              = "C:\Program Files (x86)\ossec-agent\"
$VERSION_URL             = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/$AGENT_REPO_REF/versions.json"
$STABLE_SETUP_SCRIPT_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/$AGENT_REPO_REF/scripts/windows/setup-agent.ps1"

# ---- Globals ----
$ActiveResponsesLogDir = Join-Path $OSSEC_PATH "active-response"
$LogPath               = Join-Path $ActiveResponsesLogDir "active-responses.log"
$PRERELEASE_VERSION    = $null

# ---- Logging Override ----
function Append-Log {
    param(
        [string]$Message,
        [string]$Level = "INFO"
    )

    $ts   = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
    $line = "[$ts] [$Level] $Message"

    try {
        if (-not (Test-Path $ActiveResponsesLogDir)) {
            New-Item -ItemType Directory -Force -Path $ActiveResponsesLogDir -ErrorAction Stop | Out-Null
        }

        $fileStream   = $null
        $streamWriter = $null
        try {
            $fileStream = [System.IO.FileStream]::new(
                $LogPath,
                [System.IO.FileMode]::Append,
                [System.IO.FileAccess]::Write,
                [System.IO.FileShare]::ReadWrite
            )
            $streamWriter = [System.IO.StreamWriter]::new($fileStream, [System.Text.Encoding]::UTF8)
            $streamWriter.WriteLine($line)
            $streamWriter.Flush()
        } finally {
            if ($streamWriter) { $streamWriter.Dispose() }
            if ($fileStream)   { $fileStream.Dispose() }
        }
    } catch {
        # Fallback to standard host output if log file writing fails
        Write-Host "Warning: Failed to write to log file $LogPath : $($_.Exception.Message)"
    }

    Write-Host $line
}

# ---- Helper: clean up a temp file unconditionally ----
function Remove-TempFile {
    param([string]$Path)
    if (Test-Path $Path) {
        Remove-Item $Path -Force -ErrorAction SilentlyContinue
    }
}

function Get-ActiveConsoleUser {
    # 1. Query process owner of explorer.exe via WMI (works from Session 0 as SYSTEM)
    try {
        $explorers = Get-WmiObject Win32_Process -Filter "Name='explorer.exe'" -ErrorAction SilentlyContinue
        foreach ($exp in $explorers) {
            $owner = $exp.GetOwner()
            if ($owner -and $owner.User -and $owner.User -notmatch '^(SYSTEM|LOCAL SERVICE|NETWORK SERVICE)$') {
                if ($owner.Domain) { return "$($owner.Domain)\$($owner.User)" }
                return $owner.User
            }
        }
    } catch {}

    # 2. Query via Get-CimInstance fallback
    try {
        $explorerCims = Get-CimInstance Win32_Process -Filter "Name = 'explorer.exe'" -ErrorAction SilentlyContinue
        foreach ($expCim in $explorerCims) {
            $ownerCim = Invoke-CimMethod -InputObject $expCim -MethodName GetOwner -ErrorAction SilentlyContinue
            if ($ownerCim -and $ownerCim.User -and $ownerCim.User -notmatch '^(SYSTEM|LOCAL SERVICE|NETWORK SERVICE)$') {
                if ($ownerCim.Domain) { return "$($ownerCim.Domain)\$($ownerCim.User)" }
                return $ownerCim.User
            }
        }
    } catch {}

    # 3. Environment variables fallback (when running in interactive user context)
    if ($env:USERNAME -and $env:USERNAME -notmatch '^(SYSTEM|LOCAL SERVICE|NETWORK SERVICE)$') {
        if ($env:USERDOMAIN) { return "$env:USERDOMAIN\$env:USERNAME" }
        return $env:USERNAME
    }

    # 4. Query quser for active session user
    try {
        $quser = query user 2>$null
        if ($quser) {
            foreach ($line in ($quser -split "\r?\n")[1..($quser.Count-1)]) {
                if ($line -match "Active") {
                    $fields = $line.Trim() -split "\s+"
                    $u = $fields[0].Replace(">", "")
                    if ($u -and $u -notmatch '^(SYSTEM|LOCAL SERVICE|NETWORK SERVICE)$') { return $u }
                }
            }
        }
    } catch {}

    return $null
}

function Invoke-InteractivePopup {
    param(
        [string]$Message,
        [string]$Title = "Wazuh Update",
        [string]$Mode = "Consent", # "Consent" or "Info"
        [int]$TimeoutSeconds = 600
    )
    
    # 1. Try direct WinForms GUI if running in an interactive desktop session (SessionId > 0)
    try {
        $sessionId = [System.Diagnostics.Process]::GetCurrentProcess().SessionId
        if ($sessionId -gt 0) {
            Add-Type -AssemblyName System.Windows.Forms -ErrorAction Stop | Out-Null
            Add-Type -AssemblyName System.Drawing -ErrorAction Stop | Out-Null

            if ($Mode -eq "Consent") {
                $form = New-Object System.Windows.Forms.Form
                $form.Text = $Title
                $form.Size = New-Object System.Drawing.Size(430, 180)
                $form.StartPosition = "CenterScreen"
                $form.FormBorderStyle = "FixedDialog"
                $form.MaximizeBox = $false
                $form.MinimizeBox = $false
                $form.TopMost = $true

                $label = New-Object System.Windows.Forms.Label
                $label.Text = $Message
                $label.Location = New-Object System.Drawing.Point(20, 20)
                $label.Size = New-Object System.Drawing.Size(370, 50)
                [void]$form.Controls.Add($label)

                $btnUpgrade = New-Object System.Windows.Forms.Button
                $btnUpgrade.Text = "Upgrade Now"
                $btnUpgrade.Location = New-Object System.Drawing.Point(85, 85)
                $btnUpgrade.Size = New-Object System.Drawing.Size(120, 35)
                $btnUpgrade.Add_Click({
                    $form.DialogResult = [System.Windows.Forms.DialogResult]::Yes
                    $form.Close()
                })
                [void]$form.Controls.Add($btnUpgrade)

                $btnLater = New-Object System.Windows.Forms.Button
                $btnLater.Text = "Remind Me Later"
                $btnLater.Location = New-Object System.Drawing.Point(215, 85)
                $btnLater.Size = New-Object System.Drawing.Size(130, 35)
                $btnLater.Add_Click({
                    $form.DialogResult = [System.Windows.Forms.DialogResult]::No
                    $form.Close()
                })
                [void]$form.Controls.Add($btnLater)

                $res = $form.ShowDialog()
                $form.Dispose()

                if ($res -eq [System.Windows.Forms.DialogResult]::Yes) {
                    InfoMessage "User clicked Upgrade Now."
                    return 0
                } else {
                    InfoMessage "User clicked Remind Me Later."
                    return 1
                }
            } else {
                [System.Windows.Forms.MessageBox]::Show($Message, $Title, [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Information) | Out-Null
                return 0
            }
        }
    } catch {
        $err = $_
        $errMsg = if ($err -and $err.Exception) { $err.Exception.Message } elseif ($Error[0] -and $Error[0].Exception) { $Error[0].Exception.Message } else { "Unknown error" }
        WarnMessage "Direct GUI popup failed, falling back to Scheduled Task: $errMsg"
    }

    # 2. Fallback to Scheduled Task for Session 0 (background service context)
    try {
        $consoleUser = Get-ActiveConsoleUser
        if ([string]::IsNullOrWhiteSpace($consoleUser)) {
            InfoMessage "No active desktop session found. Proceeding with background upgrade."
            return 0
        }
        
        $guid = [guid]::NewGuid().ToString('N')
        $taskName = "WazuhUpdatePopup_$guid"
        
        $pubDir = Join-Path $env:ProgramData "WazuhAgentStatus"
        if (-not (Test-Path $pubDir)) {
            New-Item -ItemType Directory -Path $pubDir -Force | Out-Null
        }
        $resultFile = Join-Path $pubDir "wazuh_popup_res_$guid.txt"

        Remove-TempFile $resultFile

        $escapedMsg = $Message.Replace("'", "''")
        $escapedTitle = $Title.Replace("'", "''")
        
        $script = @"
Add-Type -AssemblyName System.Windows.Forms | Out-Null
Add-Type -AssemblyName System.Drawing | Out-Null

try {
    if ('$Mode' -eq 'Consent') {
        `$form = New-Object System.Windows.Forms.Form
        `$form.Text = '$escapedTitle'
        `$form.Size = New-Object System.Drawing.Size(430, 180)
        `$form.StartPosition = 'CenterScreen'
        `$form.FormBorderStyle = 'FixedDialog'
        `$form.MaximizeBox = `$false
        `$form.MinimizeBox = `$false
        `$form.TopMost = `$true

        `$label = New-Object System.Windows.Forms.Label
        `$label.Text = '$escapedMsg'
        `$label.Location = New-Object System.Drawing.Point(20, 20)
        `$label.Size = New-Object System.Drawing.Size(370, 50)
        [void]`$form.Controls.Add(`$label)

        `$btnUpgrade = New-Object System.Windows.Forms.Button
        `$btnUpgrade.Text = 'Upgrade Now'
        `$btnUpgrade.Location = New-Object System.Drawing.Point(85, 85)
        `$btnUpgrade.Size = New-Object System.Drawing.Size(120, 35)
        `$btnUpgrade.Add_Click({
            `$form.DialogResult = [System.Windows.Forms.DialogResult]::Yes
            `$form.Close()
        })
        [void]`$form.Controls.Add(`$btnUpgrade)

        `$btnLater = New-Object System.Windows.Forms.Button
        `$btnLater.Text = 'Remind Me Later'
        `$btnLater.Location = New-Object System.Drawing.Point(215, 85)
        `$btnLater.Size = New-Object System.Drawing.Size(130, 35)
        `$btnLater.Add_Click({
            `$form.DialogResult = [System.Windows.Forms.DialogResult]::No
            `$form.Close()
        })
        [void]`$form.Controls.Add(`$btnLater)

        `$res = `$form.ShowDialog()
        `$form.Dispose()
    } else {
        `$res = [System.Windows.Forms.MessageBox]::Show('$escapedMsg', '$escapedTitle', [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Information)
    }

    if (`$res -eq [System.Windows.Forms.DialogResult]::Yes -or `$res -eq [System.Windows.Forms.DialogResult]::OK) {
        Set-Content -Path '$resultFile' -Value 'Yes' -Force
    } else {
        Set-Content -Path '$resultFile' -Value 'No' -Force
    }
} catch {
    Set-Content -Path '$resultFile' -Value "Error: `$($_.Exception.Message)" -Force
}
"@
        $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($script))
        
        $action = New-ScheduledTaskAction -Execute "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe" -Argument "-NoProfile -EncodedCommand $encoded"
        $principal = New-ScheduledTaskPrincipal -UserId $consoleUser -LogonType Interactive
        $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -Priority 4
        
        Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal -Settings $settings -Force | Out-Null
        Start-ScheduledTask -TaskName $taskName | Out-Null
        
        $elapsed = 0
        while (-not (Test-Path $resultFile)) {
            if ($elapsed -ge $TimeoutSeconds) {
                WarnMessage "Popup timed out after $TimeoutSeconds seconds."
                Stop-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue | Out-Null
                Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue | Out-Null
                Remove-TempFile $resultFile
                return 1
            }
            Start-Sleep -Seconds 1
            $elapsed++
        }
        
        $userChoice = (Get-Content -Path $resultFile -Raw).Trim()
        Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue | Out-Null
        Remove-TempFile $resultFile
        
        if ($userChoice -eq "Yes") {
            return 0
        } else {
            InfoMessage "Popup result: $userChoice"
            return 1
        }
    } catch {
        $err = $_
        $errMsg = if ($err -and $err.Exception) { $err.Exception.Message } elseif ($Error[0] -and $Error[0].Exception) { $Error[0].Exception.Message } else { "Unknown error" }
        WarnMessage "Interactive popup failed: $errMsg"
        return 1
    }
}

function Get-UserConsent {
    InfoMessage "Prompting user for upgrade consent..."
    $msg = "A new version of Wazuh is available. Would you like to upgrade?"
    $res = Invoke-InteractivePopup -Message $msg -Mode "Consent" -TimeoutSeconds 3600
    if ($res -ne 0) {
        InfoMessage "User chose Remind Me Later or prompt timed out."
        return $false
    }
    InfoMessage "User chose to Upgrade Now."
    return $true
}

function Get-PrereleaseVersion {
    try {
        InfoMessage "Fetching prerelease version from: $VERSION_URL"
        $response = Invoke-RestMethod -Uri $VERSION_URL -Method Get -TimeoutSec 30

        if ($response -and $response.framework -and $response.framework.prerelease_version) {
            $version = $response.framework.prerelease_version
            InfoMessage "Successfully fetched prerelease version: $version"
            return $version
        } else {
            WarnMessage "No prerelease version found in response."
            return $null
        }
    } catch {
        WarnMessage "Failed to fetch prerelease version: $($_.Exception.Message)"
        return $null
    }
}

function Run-Update {
    InfoMessage "Starting Wazuh agent upgrade..."
    InfoMessage "Using temporary directory: $env:TEMP"

    # Determine setup script URL without shadowing the module-level constant
    if ($Prerelease) {
        $resolvedScriptUrl = $PRERELEASE_SETUP_SCRIPT_URL
        InfoMessage "Using prerelease setup script: $resolvedScriptUrl"
    } else {
        $resolvedScriptUrl = $STABLE_SETUP_SCRIPT_URL
        InfoMessage "Using stable setup script: $resolvedScriptUrl"
    }

    $setupScriptPath = Join-Path $env:TEMP "setup-agent.ps1"

    InfoMessage "Downloading setup script..."
    try {
        Invoke-WebRequest -Uri $resolvedScriptUrl -OutFile $setupScriptPath -ErrorAction Stop
    } catch {
        ErrorMessage "Failed to download setup-agent.ps1: $($_.Exception.Message)"
        exit 1
    }

    InfoMessage "Executing setup script: $setupScriptPath"
    $env:WAZUH_MANAGER = $WAZUH_MANAGER
    # Tell install.ps1 this is an update so it leaves the running server
    # service (the one executing this chain) alone instead of stopping it.
    $env:WAZUH_AGENT_STATUS_UPDATE = "1"
    try {
        & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $setupScriptPath
        if ($LASTEXITCODE -ne 0) {
            ErrorMessage "Setup script failed (exit code: $LASTEXITCODE)."
            exit 1
        }
    } catch {
        ErrorMessage "Failed to execute setup script: $($_.Exception.Message)"
        exit 1
    } finally {
        Remove-TempFile $setupScriptPath
    }

    SuccessMessage "Update completed successfully! Please save your work and reboot to finish the update."
    # Fire and forget success popup (timeout short so it doesn't block indefinitely)
    Invoke-InteractivePopup -Message "Update completed successfully! Please save your work and reboot to finish the update." -Buttons "OK" -Icon "Information" -TimeoutSeconds 60 | Out-Null
}

# ---- Main Execution ----
InfoMessage "Wazuh Agent Upgrade Script"
InfoMessage "Running as Administrator: $IsAdmin"
InfoMessage "Log file: $LogPath"

if (-not (Get-UserConsent)) {
    InfoMessage "Update postponed. Exiting."
    exit 0
}

if ($Prerelease) {
    $PRERELEASE_VERSION = Get-PrereleaseVersion
    if ($PRERELEASE_VERSION) {
        InfoMessage "PRERELEASE UPGRADE MODE: Installing prerelease version $PRERELEASE_VERSION"
        $PRERELEASE_SETUP_SCRIPT_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v$PRERELEASE_VERSION/scripts/windows/setup-agent.ps1"
        # Point setup-agent.ps1 at the same tag so it downloads its components
        # and version.txt from the prerelease release.
        $env:WAZUH_AGENT_REPO_REF = "refs/tags/v$PRERELEASE_VERSION"
    } else {
        WarnMessage "Failed to fetch prerelease version. Exiting."
        exit 1
    }
} else {
    InfoMessage "STABLE UPGRADE MODE: Installing latest stable version."
}

InfoMessage "Starting upgrade process..."
Run-Update
InfoMessage "Script execution completed."
