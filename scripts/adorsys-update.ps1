param (
    [string]$WazuhManager = "wazuh.example.com",
    [string]$ScriptUrl = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/heads/fix/agent-update-binary/scripts/setup-agent.ps1"
)

$ErrorActionPreference = "Stop"
$VerbosePreference = "Continue"
$DebugPreference = "Continue"

# Paths
$LogFile = "${env:ProgramFiles(x86)}\ossec-agent\active-response\active-responses.log"
$TmpFolder = New-Item -ItemType Directory -Path ([System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()) -Force

# Start transcript for full command output logging
$TranscriptPath = Join-Path $TmpFolder "transcript.log"
Start-Transcript -Path $TranscriptPath -Append | Out-Null

function Cleanup {
    if (Test-Path $TmpFolder) {
        Remove-Item -Recurse -Force $TmpFolder
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


# Non-interactive mode: default user action to Yes
$UserAction = 'Yes'

function Run-Upgrade {
    InfoMessage "Starting Wazuh agent upgrade..."
    InfoMessage "Using temporary directory: $TmpFolder"
    $ServerServiceName = "wazuh-agent-status"
    $ClientServiceName = "wazuh-agent-status-client"

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
    $SetupScript = Join-Path $TmpFolder "setup-agent.ps1"
    try {
        Invoke-WebRequest -Uri $ScriptUrl -OutFile $SetupScript -UseBasicParsing -Verbose *>> $LogFile 2>&1
    } catch {
        ErrorMessage "Failed to download setup-agent.ps1" $_.Exception
        exit 1
    }

    InfoMessage "Stopping services..."
    try {
        InfoMessage "Stopping $ServerServiceName service..."
        Stop-Service -Name $ServerServiceName -ErrorAction Stop -Verbose *>> $LogFile 2>&1
    } catch {
        ErrorMessage "Failed to stop $ServerServiceName service" $_.Exception
        exit 1
    }

    try {
        InfoMessage "Stopping $ClientServiceName process..."
        $ClientServiceProc = Get-Process -Name $ClientServiceName -ErrorAction SilentlyContinue
        if ($ClientServiceProc) {
            Stop-Process -Id $ClientServiceProc.Id -Force -ErrorAction Stop -Verbose *>> $LogFile 2>&1
        } else {
            InfoMessage "Client process not running or already stopped."
        }
    } catch {
        ErrorMessage "Failed to stop $ClientServiceName process" $_.Exception
        exit 1
    }

    try {
        InfoMessage "Running setup-agent.ps1..."
        $env:WAZUH_MANAGER = $WazuhManager

        # Execute external setup script, redirect all output to log
        & powershell -ExecutionPolicy Bypass -File $SetupScript -WazuhManager $WazuhManager *>> $LogFile 2>&1

        if ($LASTEXITCODE -ne 0) {
            throw "setup-agent.ps1 exited with code $LASTEXITCODE"
        }

        InfoMessage "setup-agent.ps1 executed successfully."
    } catch {
        ErrorMessage "Failed to setup wazuh agent" $_.Exception
        exit 1
    }

    InfoMessage "Update completed successfully. Please save your work and reboot your device to complete the update."
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