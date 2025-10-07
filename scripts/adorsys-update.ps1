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

function Show-UserNotification {
    <#
    .SYNOPSIS
        Shows a notification to the active user via scheduled task
    .DESCRIPTION
        Creates a temporary scheduled task that runs in the logged-in user's session
        to display a toast notification. Works from background/service contexts.
    #>
    param (
        [Parameter(Mandatory=$true)]
        [string]$Title,

        [Parameter(Mandatory=$true)]
        [string]$Message,

        [Parameter(Mandatory=$false)]
        [switch]$IsError
    )

    try {
        InfoMessage "Attempting to show notification: $Title - $Message"

        # Generate unique task name
        $TaskName = "WazuhNotification_" + [Guid]::NewGuid().ToString().Substring(0, 8)
        $TempScript = Join-Path $env:TEMP "$TaskName.ps1"

        # Get active console user
        $ActiveUser = $null
        try {
            $sessions = query user 2>$null | Select-Object -Skip 1
            foreach ($session in $sessions) {
                if ($session -match '^\s*>?(\S+)\s+console\s+(\d+)\s+Active') {
                    $ActiveUser = $matches[1]
                    InfoMessage "Found active console user: $ActiveUser"
                    break
                }
            }
        } catch {
            WarningMessage "Failed to query user sessions: $($_.Exception.Message)"
        }

        # Fallback to current user if no active session found
        if (-not $ActiveUser) {
            $ActiveUser = [System.Environment]::UserName
            if (-not $ActiveUser -or $ActiveUser -eq "SYSTEM") {
                # Try getting from registry
                $LoggedOnUser = Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\LogonUI" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty LastLoggedOnUser -ErrorAction SilentlyContinue
                if ($LoggedOnUser -and $LoggedOnUser -match '\\(.+)$') {
                    $ActiveUser = $matches[1]
                }
            }
            InfoMessage "Using fallback user: $ActiveUser"
        }

        if (-not $ActiveUser -or $ActiveUser -eq "SYSTEM") {
            WarningMessage "No active user found, notification cannot be displayed"
            return $false
        }

        # Create notification script with logging
        $NotificationLog = Join-Path $env:TEMP "$TaskName.log"
        $ScriptContent = @"
# Notification script for Wazuh Update
`$LogFile = '$($NotificationLog -replace "'", "''")'
`$Timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'

# Log function
function LogNotif(`$msg) {
    "`$Timestamp - `$msg" | Out-File -FilePath `$LogFile -Append -Encoding UTF8
}

LogNotif "Notification script started for user: $ActiveUser"

try {
    # Try BurntToast first (modern Windows 10/11)
    if (Get-Module -ListAvailable -Name BurntToast) {
        LogNotif "BurntToast module found, importing..."
        Import-Module BurntToast -ErrorAction Stop
        `$params = @{
            Text = '$($Title -replace "'", "''")', '$($Message -replace "'", "''")'
            Sound = $(if ($IsError) { "'Alarm'" } else { "'Default'" })
        }
        LogNotif "Sending BurntToast notification..."
        New-BurntToastNotification @params
        LogNotif "BurntToast notification sent successfully"
    } else {
        LogNotif "BurntToast module not found, using msg.exe fallback"
        # Fallback to msg.exe for older systems
        # Find user session ID
        `$sessions = query user 2>&1
        LogNotif "Query user output: `$sessions"
        `$sessionId = (`$sessions | Where-Object { `$_ -match '$ActiveUser' } | ForEach-Object {
            if (`$_ -match '\s+(\d+)\s+Active') {
                `$matches[1]
            }
        }) | Select-Object -First 1

        if (`$sessionId) {
            LogNotif "Found session ID: `$sessionId, sending msg.exe..."
            msg.exe `$sessionId /TIME:30 "$Title - $Message" 2>&1 | Out-String | ForEach-Object { LogNotif "msg.exe output: `$_" }
            LogNotif "msg.exe notification sent"
        } else {
            LogNotif "ERROR: Could not find active session for user $ActiveUser"
        }
    }
} catch {
    LogNotif "ERROR: Exception occurred: `$(`$_.Exception.Message)"
    LogNotif "ERROR: Stack trace: `$(`$_.ScriptStackTrace)"
}

# Cleanup with delay to ensure notification displays
LogNotif "Waiting 5 seconds before cleanup..."
Start-Sleep -Seconds 5
LogNotif "Starting cleanup..."
Unregister-ScheduledTask -TaskName '$TaskName' -Confirm:`$false -ErrorAction SilentlyContinue
Remove-Item -Path '$TempScript' -Force -ErrorAction SilentlyContinue
LogNotif "Cleanup completed"
"@

        # Write script to temp file
        $ScriptContent | Out-File -FilePath $TempScript -Encoding UTF8 -Force

        # Create scheduled task using schtasks.exe to avoid XML validation issues
        $StartTime = (Get-Date).AddSeconds(5).ToString("HH:mm")

        InfoMessage "Creating scheduled task with schtasks.exe..."
        $createOutput = schtasks.exe /Create /F /SC ONCE /TN "$TaskName" /TR "powershell.exe -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$TempScript`"" /ST $StartTime /RU "$ActiveUser" 2>&1
        InfoMessage "Schtasks create output: $createOutput"

        # Wait a moment then start the task immediately
        Start-Sleep -Milliseconds 500
        InfoMessage "Starting scheduled task immediately..."
        $runOutput = schtasks.exe /Run /TN "$TaskName" 2>&1
        InfoMessage "Schtasks run output: $runOutput"

        InfoMessage "Notification task created successfully: $TaskName. Check log file: $NotificationLog"
        return $true

    } catch {
        WarningMessage "Failed to create notification task: $($_.Exception.Message)"
        return $false
    }
}

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
        Show-UserNotification -Title "Wazuh Update Failed" -Message "Update failed: Invoke-WebRequest is missing. See log file: $LogFile" -IsError
        exit 1
    }
    if (-not (Get-Command "powershell" -ErrorAction SilentlyContinue)) {
        ErrorMessage "PowerShell is required but not available."
        Show-UserNotification -Title "Wazuh Update Failed" -Message "Update failed: PowerShell is missing. See log file: $LogFile" -IsError
        exit 1
    }

    InfoMessage "Downloading setup script..."
    $SetupScript = Join-Path $TmpFolder "setup-agent.ps1"
    try {
        Invoke-WebRequest -Uri $ScriptUrl -OutFile $SetupScript -UseBasicParsing -Verbose *>> $LogFile 2>&1
    } catch {
        ErrorMessage "Failed to download setup-agent.ps1" $_.Exception
        Show-UserNotification -Title "Wazuh Update Failed" -Message "Update failed: Could not download setup script. See log file: $LogFile" -IsError
        exit 1
    }

    InfoMessage "Stopping services..."
    try {
        InfoMessage "Stopping $ServerServiceName service..."
        Stop-Service -Name $ServerServiceName -ErrorAction Stop -Verbose *>> $LogFile 2>&1
    } catch {
        ErrorMessage "Failed to stop $ServerServiceName service" $_.Exception
        Show-UserNotification -Title "Wazuh Update Failed" -Message "Update failed: Could not stop $ServerServiceName service. See log file: $LogFile" -IsError
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
        Show-UserNotification -Title "Wazuh Update Failed" -Message "Update failed: Could not stop $ClientServiceName process. See log file: $LogFile" -IsError
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
        Show-UserNotification -Title "Wazuh Update Failed" -Message "Update failed: Setup script encountered an error. See log file: $LogFile" -IsError
        exit 1
    }

    InfoMessage "Update completed successfully. Please save your work and reboot your device to complete the update."
    Show-UserNotification -Title "Wazuh Update Completed" -Message "Update completed successfully! Please save your work and reboot your device to complete the update."
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