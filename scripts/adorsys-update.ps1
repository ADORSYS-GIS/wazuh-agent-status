param (
    [string]$WazuhManager = "wazuh.example.com",
    [string]$ScriptUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/main/scripts/setup-agent.ps1"
)

$ErrorActionPreference = "Stop"

# Paths
$APP_DATA = "C:\ProgramData\ossec-agent\"
$IconPath = Join-Path -Path $APP_DATA -ChildPath "wazuh-logo.png"
$LogFile = "${env:ProgramFiles(x86)}\ossec-agent\active-response\active-responses.log"
$TmpFolder = New-Item -ItemType Directory -Path ([System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()) -Force

function Cleanup {
    if (Test-Path $TmpFolder) {
        Remove-Item -Recurse -Force $TmpFolder
    }
}
Register-EngineEvent PowerShell.Exiting -Action { Cleanup }

function Log {
    param (
        [string]$Level,
        [string]$Message
    )
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    "$timestamp [$Level] $Message" | Out-File -FilePath $LogFile -Append -Encoding UTF8
}

function InfoMessage { param($msg) Log "INFO" $msg }
function WarningMessage { param($msg) Log "WARNING" $msg }
function ErrorMessage { param($msg) Log "ERROR" $msg }

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

Add-Type -AssemblyName System.Windows.Forms

# === Notify User with Action Dialog ===
$PrepareMsg = "A new version of Wazuh is available. Would you like to upgrade?"
$UserAction = [System.Windows.Forms.MessageBox]::Show($PrepareMsg, "Wazuh Update", [System.Windows.Forms.MessageBoxButtons]::YesNo, [System.Windows.Forms.MessageBoxIcon]::Question)

function Run-Upgrade {
    InfoMessage "Starting Wazuh agent upgrade..."
    InfoMessage "Using temporary directory: $TmpFolder"

    # Check for required dependencies
    if (-not (Get-Command "Invoke-WebRequest" -ErrorAction SilentlyContinue)) {
        ErrorMessage "Invoke-WebRequest is required but not available."
        Send-Notification "Update failed: Invoke-WebRequest is missing."
        exit 1
    }
    if (-not (Get-Command "powershell" -ErrorAction SilentlyContinue)) {
        ErrorMessage "PowerShell is required but not available."
        Send-Notification "Update failed: PowerShell is missing."
        exit 1
    }

    InfoMessage "Downloading setup script..."
    $SetupScript = Join-Path $TmpFolder "setup-agent.ps1"
    try {
        Invoke-WebRequest -Uri $ScriptUrl -OutFile $SetupScript -UseBasicParsing
    } catch {
        ErrorMessage "Failed to download setup-agent.ps1"
        Send-Notification "Update failed: For more details go to file $LogFile"
        exit 1
    }

    try {
        InfoMessage "Running setup-agent.ps1..."
        $env:WAZUH_MANAGER = $WazuhManager
        & powershell -ExecutionPolicy Bypass -File $SetupScript -WazuhManager $WazuhManager *>> $LogFile
    } catch {
        ErrorMessage "Failed to setup wazuh agent"
        Send-Notification "Update failed: For more details go to file $LogFile"
        exit 1
    }

    Send-Notification "Update completed successfully! Please save your work and reboot your device to complete the update."
}

InfoMessage "Wazuh agent upgrade script started."

switch ($UserAction) {
    'Yes' {
        InfoMessage "User chose to update now."
        Run-Upgrade
        exit 0
    }
    'No' {
        InfoMessage "Update postponed. Exiting."
        exit 0
    }
    default {
        InfoMessage "Update postponed. Exiting."
        exit 0
    }
}