param (
    [string]$WazuhManager = "wazuh.example.com",
    [string]$ScriptUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/heads/fix/agent-update-binary/scripts/setup-agent.ps1"
)

$ErrorActionPreference = "Stop"
$VerbosePreference = "Continue"
$DebugPreference = "Continue"

# Paths
$LogFile = "${env:ProgramFiles(x86)}\ossec-agent\active-response\active-responses.log"
$StatusFile = "C:\ProgramData\WazuhAgent\update_status.json"
$TmpFolder = New-Item -ItemType Directory -Path ([System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()) -Force

# Start transcript for full command output logging
$TranscriptPath = Join-Path $TmpFolder "transcript.log"
Start-Transcript -Path $TranscriptPath -Append | Out-Null

function Cleanup {
    if (Test-Path $TmpFolder) {
        Remove-Item -Recurse -Force $TmpFolder
    }
    # Clean up status file on exit
    if (Test-Path $StatusFile) {
        Remove-Item -Force $StatusFile -ErrorAction SilentlyContinue
    }
}
Register-EngineEvent PowerShell.Exiting -Action { Cleanup }

function Log {
    param (
        [string]$Level,
        [string]$Message,
        [System.Exception]$Exception = $null
    )
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logMessage = "$timestamp [$Level] $Message"
    if ($Exception) {
        $logMessage += "`nException: $($Exception.ToString())"
    }
    $logMessage | Out-File -FilePath $LogFile -Append -Encoding UTF8
}

function InfoMessage { param($msg) Log "INFO" $msg }
function WarningMessage { param($msg) Log "WARNING" $msg }
function ErrorMessage { param($msg, $ex = $null) Log "ERROR" $msg $ex }

function Write-UpdateStatus {
    param(
        [string]$Status,
        [string]$Message = ""
    )
    $statusObj = @{
        status = $Status
        message = $Message
        timestamp = (Get-Date).ToString("o")
    }
    $statusJson = $statusObj | ConvertTo-Json -Compress
    $statusJson | Out-File -FilePath $StatusFile -Encoding UTF8 -Force
    InfoMessage "Status updated: $Status"
}

function Send-Notification {
    param (
        [string]$Message,
        [string]$Title = "Wazuh Update"
    )
    try {
        # Use BurntToast for notifications
        if (-not (Get-Module -ListAvailable -Name BurntToast)) {
            Install-Module -Name BurntToast -Force -Scope CurrentUser -AllowClobber
        }
        Import-Module BurntToast -Force
        if (Test-Path $IconPath) {
            New-BurntToastNotification -Text $Title, $Message -AppLogo $IconPath
        } else {
            New-BurntToastNotification -Text $Title, $Message
        }
        InfoMessage "Notification sent: $Message"
    } catch {
        # Fallback to message box
        [System.Windows.Forms.MessageBox]::Show($Message, $Title, [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Information) | Out-Null
        InfoMessage "Notification sent (fallback): $Message"
    }
}

# Non-interactive mode: default user action to Yes
$UserAction = 'Yes'

function Run-Upgrade {
    InfoMessage "Starting Wazuh agent upgrade..."
    InfoMessage "Using temporary directory: $TmpFolder"
    Write-UpdateStatus -Status "started"

    # Check for required dependencies
    if (-not (Get-Command "Invoke-WebRequest" -ErrorAction SilentlyContinue)) {
        ErrorMessage "Invoke-WebRequest is required but not available."
        exit 1
    }
    if (-not (Get-Command "powershell" -ErrorAction SilentlyContinue)) {
        ErrorMessage "PowerShell is required but not available."
        exit 1
    }

    InfoMessage "Downloading setup script..."
    Write-UpdateStatus -Status "downloading"
    $SetupScript = Join-Path $TmpFolder "setup-agent.ps1"
    try {
        Invoke-WebRequest -Uri $ScriptUrl -OutFile $SetupScript -UseBasicParsing -Verbose *>> $LogFile 2>&1
    } catch {
        ErrorMessage "Failed to download setup-agent.ps1"
        Write-UpdateStatus -Status "error" -Message "Failed to download setup script"
        Send-Notification "Update failed: For more details go to file $LogFile"
        exit 1
    }

    try {
        InfoMessage "Running setup-agent.ps1..."
        Write-UpdateStatus -Status "installing"
        $env:WAZUH_MANAGER = $WazuhManager

        # Execute external setup script, redirect all output to log
        & powershell -ExecutionPolicy Bypass -File $SetupScript -WazuhManager $WazuhManager *>> $LogFile 2>&1

        if ($LASTEXITCODE -ne 0) {
            throw "setup-agent.ps1 exited with code $LASTEXITCODE"
        }

        InfoMessage "setup-agent.ps1 executed successfully."
    } catch {
        ErrorMessage "Failed to setup wazuh agent"
        Write-UpdateStatus -Status "error" -Message "Failed to install Wazuh agent"
        Send-Notification "Update failed: For more details go to file $LogFile"
        exit 1
    }

    Write-UpdateStatus -Status "success"
    Send-Notification "Update completed successfully! Please save your work and reboot your device to complete the update."
}

InfoMessage "Wazuh agent upgrade script started."

switch ($UserAction) {
    'Yes' {
        InfoMessage "User chose to update now."
        Run-Upgrade
        Stop-Transcript | Out-Null
        exit 0
    }
    'No' {
        InfoMessage "Update postponed. Exiting."
        Stop-Transcript | Out-Null
        exit 0
    }
    default {
        InfoMessage "Update postponed. Exiting."
        Stop-Transcript | Out-Null
        exit 0
    }
}