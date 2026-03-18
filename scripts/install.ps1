# Ensure the script is running as administrator
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "This script must be run as Administrator."
    exit 1
}

# Default Variables
$WAZUH_MANAGER = if ($null -ne $env:WAZUH_MANAGER) { $env:WAZUH_MANAGER } else { "wazuh.example.com" }
$SERVER_NAME = if ($null -ne $env:SERVER_NAME) { $env:SERVER_NAME } else { "wazuh-agent-status" }
$CLIENT_NAME = if ($null -ne $env:CLIENT_NAME) { $env:CLIENT_NAME } else { "wazuh-agent-status-client" }
$INSTALL_PROFILE = if ($null -ne $env:INSTALL_PROFILE) { $env:INSTALL_PROFILE } else { "user" }

$APP_VERSION = if ($null -ne $env:APP_VERSION) { $env:APP_VERSION } else { "0.4.2" }

if ($INSTALL_PROFILE -eq "admin") {
    $WAS_VERSION = $APP_VERSION
} else {
    $WAS_VERSION = "$APP_VERSION-user"
}

# Determine architecture and operating system
$ARCH = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { "amd32" }
$BIN_DIR = "C:\Program Files\$SERVER_NAME"
$SERVER_EXE = "$BIN_DIR\$SERVER_NAME.exe"
$CLIENT_EXE = "$BIN_DIR\$CLIENT_NAME.exe"

$UPDATE_SCRIPT_URL = if ($null -ne $env:UPDATE_SCRIPT_URL) { $env:UPDATE_SCRIPT_URL } else { "https://raw.githubusercontent.com/ADORSYS-GIS/$SERVER_NAME/v$WAS_VERSION/scripts/adorsys-update.bat" }
$UPDATE_SCRIPT_PATH = if ($null -ne $env:UPDATE_SCRIPT_PATH) { $env:UPDATE_SCRIPT_PATH } else { "${env:ProgramFiles(x86)}\ossec-agent\active-response\bin\adorsys-update.bat" }

$UPDATE_SCRIPT_PS_URL = if ($null -ne $env:UPDATE_SCRIPT_PS_URL) { $env:UPDATE_SCRIPT_PS_URL } else { "https://raw.githubusercontent.com/ADORSYS-GIS/$SERVER_NAME/v$WAS_VERSION/scripts/adorsys-update.ps1" } 
$UPDATE_SCRIPT_PS_PATH = if ($null -ne $env:UPDATE_SCRIPT_PS_PATH) { $env:UPDATE_SCRIPT_PS_PATH } else { "${env:ProgramFiles(x86)}\ossec-agent\active-response\bin\adorsys-update.ps1" }

# Create necessary directories
if (-not (Test-Path $BIN_DIR)) {
    New-Item -Path $BIN_DIR -ItemType Directory | Out-Null
}

# Function for logging with timestamp
function Log {
    param (
        [string]$Level,
        [string]$Message
    )
    $Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "$Timestamp $Level $Message"
}

# Logging helpers
function InfoMessage { param ([string]$Message) Log "[INFO]" $Message }
function WarnMessage { param ([string]$Message) Log "[WARNING]" $Message }
function ErrorMessage { param ([string]$Message) Log "[ERROR]" $Message }
function SuccessMessage { param ([string]$Message) Log "[SUCCESS]" $Message }
function PrintStep { param ([int]$StepNumber, [string]$Message) Log "[STEP]" "Step ${StepNumber}: $Message" }

# Exit script with an error message
function ErrorExit {
    param ([string]$Message)
    ErrorMessage $Message
    exit 1
}

function Validate-Installation {
    PrintStep 6 "Validating installation and configuration..."

    # Validate server binary
    if (Test-Path -LiteralPath $SERVER_EXE) {
        SuccessMessage "Server binary exists: $SERVER_EXE."
    } else {
        ErrorExit "Server binary is missing: $SERVER_EXE."
    }

    # Validate client binary
    if (Test-Path -LiteralPath $CLIENT_EXE) {
        SuccessMessage "Client binary exists: $CLIENT_EXE."
    } else {
        ErrorExit "Client binary is missing: $CLIENT_EXE."
    }

    # Validate Windows service
    try {
        $service = Get-Service -Name $SERVER_NAME -ErrorAction Stop
        SuccessMessage "Windows service exists: $SERVER_NAME."

        if ($service.Status -eq 'Running') {
            SuccessMessage "Windows service is running: $SERVER_NAME."
        } else {
            ErrorExit "Windows service is not running: $SERVER_NAME (current status: $($service.Status))."
        }
    } catch {
        ErrorExit "Windows service is missing: $SERVER_NAME."
    }

    # Validate startup shortcut for client
    $startupShortcutPath = [System.IO.Path]::Combine($env:APPDATA, "Microsoft\Windows\Start Menu\Programs\Startup", "$CLIENT_NAME.lnk")
    if (Test-Path -LiteralPath $startupShortcutPath) {
        SuccessMessage "Startup shortcut exists: $startupShortcutPath."
    } else {
        ErrorExit "Startup shortcut is missing: $startupShortcutPath."
    }

    # Validate adorsys-update script
    if (Test-Path -LiteralPath $UPDATE_SCRIPT_PATH) {
        SuccessMessage "adorsys-update script exists: $UPDATE_SCRIPT_PATH."
    } else {
        ErrorExit "adorsys-update script is missing: $UPDATE_SCRIPT_PATH."
    }

    # Validate adorsys-update PowerShell script
    if (Test-Path -LiteralPath $UPDATE_SCRIPT_PS_PATH) {
        SuccessMessage "adorsys-update PowerShell script exists: $UPDATE_SCRIPT_PS_PATH."
    } else {
        ErrorExit "adorsys-update PowerShell script is missing: $UPDATE_SCRIPT_PS_PATH."
    }

    SuccessMessage "Installation validation completed successfully."
}

function Download-File {
    param(
        [string]$Url,
        [string]$OutputPath
    )
    try {
        Invoke-WebRequest -Uri $Url -OutFile $OutputPath -ErrorAction Stop
        InfoMessage "Downloaded $OutputPath from $Url."
    } catch {
        ErrorExit "Failed to download $Url."
    }
}

function Create-Service {
    param(
        [string]$ServiceName,
        [string]$ExecutablePath,
        [string]$DisplayName = $null,
        [string]$Description = $null
    )
    $ServiceExists = Get-WmiObject -Class Win32_Service -Filter "Name='$ServiceName'" -ErrorAction SilentlyContinue

    if ($ServiceExists) {
        InfoMessage "Service $ServiceName already exists. Updating..."
        Stop-Service -Name $ServiceName -Force
        sc.exe delete $ServiceName
        Start-Sleep -Seconds 3
    }

    InfoMessage "Creating service $ServiceName..."
    sc.exe create $ServiceName binPath= "`"$ExecutablePath`"" start= auto DisplayName= "`"$DisplayName`"" obj= "LocalSystem"
    sc.exe description $ServiceName "$Description"

    # Start the service
    try {
        Start-Service -Name $ServiceName
        InfoMessage "Service $ServiceName created and started successfully."
    } catch {
        ErrorMessage "Failed to start service $ServiceName. Check service logs for more information."
    }
}

function Create-StartupShortcut {
    param(
        [string]$ShortcutName,
        [string]$ExecutablePath
    )
    $ShortcutPath = [System.IO.Path]::Combine($env:APPDATA, "Microsoft\Windows\Start Menu\Programs\Startup", "$ShortcutName.lnk")
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = $ExecutablePath
    $Shortcut.Save()
    InfoMessage "Startup shortcut created: $ShortcutPath."
}

# Download binaries
$BaseURL = "https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/download/v$WAS_VERSION"
$ServerURL = "$BaseURL/$SERVER_NAME-windows-$ARCH.exe"
$ClientURL = "$BaseURL/$CLIENT_NAME-windows-$ARCH.exe"

PrintStep 1 "Stopping existing agent-status service and client processes..."
try {
    # Stop the service if it exists
    $Service = Get-Service -Name $SERVER_NAME -ErrorAction SilentlyContinue
    if ($Service) {
        if ($Service.Status -eq 'Running') {
            InfoMessage "Stopping $SERVER_NAME service..."
            Stop-Service -Name $SERVER_NAME -Force -ErrorAction Stop
            InfoMessage "Service $SERVER_NAME stopped successfully."
        } else {
            InfoMessage "Service $SERVER_NAME is not running."
        }
    } else {
        InfoMessage "Service $SERVER_NAME does not exist."
    }

    # Stop any running client processes
    $ClientProcesses = Get-Process -Name $CLIENT_NAME -ErrorAction SilentlyContinue
    if ($ClientProcesses) {
        InfoMessage "Stopping $CLIENT_NAME processes..."
        $ClientProcesses | ForEach-Object {
            Stop-Process -Id $_.Id -Force
        }
        InfoMessage "All $CLIENT_NAME processes stopped successfully."
    } else {
        InfoMessage "No running $CLIENT_NAME processes found."
    }

    # Wait a moment for processes to fully terminate
    Start-Sleep -Seconds 2
} catch {
    WarnMessage "Error while stopping existing services/processes: $($_.Exception.Message)"
    WarnMessage "Continuing with installation..."
}

PrintStep 2 "Downloading binaries..."
Download-File -Url $ServerURL -OutputPath "$BIN_DIR\$SERVER_NAME.exe"
Download-File -Url $ClientURL -OutputPath "$BIN_DIR\$CLIENT_NAME.exe"

# Configure server as a Windows service
PrintStep 3 "Configuring server service..."
Create-Service -ServiceName $SERVER_NAME -ExecutablePath $SERVER_EXE -DisplayName "Wazuh Agent Status Server" -Description "Wazuh Agent Status monitoring server."

# Add client to Windows startup
PrintStep 4 "Configuring client startup..."
Create-StartupShortcut -ShortcutName $CLIENT_NAME -ExecutablePath $CLIENT_EXE

# Download adorsys-update script
PrintStep 5 "Downloading adorsys-update scripts..."
# Check if the script is currently running
$UpdateProcesses = Get-Process -Name "adorsys-update" -ErrorAction SilentlyContinue
if ($UpdateProcesses) {
    InfoMessage "adorsys-update.bat is currently running. Downloading to .new file for delayed replacement..."
    $UpdateScriptNewPath = "$UPDATE_SCRIPT_PATH.new"
    Download-File -Url $UPDATE_SCRIPT_URL -OutputPath $UpdateScriptNewPath
    InfoMessage "New version downloaded to: $UpdateScriptNewPath"
    InfoMessage "Creating scheduled task to replace script on next reboot..."

    # Also download PowerShell script
    $UpdateScriptPsNewPath = "$UPDATE_SCRIPT_PS_PATH.new"
    Download-File -Url $UPDATE_SCRIPT_PS_URL -OutputPath $UpdateScriptPsNewPath
    InfoMessage "PowerShell version downloaded to: $UpdateScriptPsNewPath"

    # Create a scheduled task to replace the script after logon
    $TaskName = "AdorsysUpdateSwap"
    $SwapScriptPath = "C:\ProgramData\ossec-agent\Run-UpdateSwap.ps1"
    $SwapScript = @"
#Requires -Version 5.1
`$ErrorActionPreference = 'Stop'

`$updateScriptPath    = '$UPDATE_SCRIPT_PATH'
`$updateScriptNewPath = '$UPDATE_SCRIPT_PATH.new'
`$updateScriptOldPath = '$UPDATE_SCRIPT_PATH.old'
`$updateScriptPsPath    = '$UPDATE_SCRIPT_PS_PATH'
`$updateScriptPsNewPath = '$UPDATE_SCRIPT_PS_PATH.new'
`$updateScriptPsOldPath = '$UPDATE_SCRIPT_PS_PATH.old'
`$logPath          = 'C:\Program Files (x86)\ossec-agent\active-response\active-responses.log'

function Write-SwapLog {
    param([string]`$Message)
    try {
        `$timestamp  = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
        `$logMessage = "[`$timestamp] [UPDATE-SWAP] `$Message"
        Add-Content -Path `$logPath -Value `$logMessage -ErrorAction SilentlyContinue
    } catch {}
}

Write-SwapLog 'Update swap task started'

try {
    if (Test-Path -LiteralPath `$updateScriptNewPath) {
        Write-SwapLog 'Found pending update for .bat script'

        if (Test-Path -LiteralPath `$updateScriptOldPath) {
            Remove-Item -LiteralPath `$updateScriptOldPath -Force
            Write-SwapLog 'Removed old backup for .bat script'
        }

        if (Test-Path -LiteralPath `$updateScriptPath) {
            Move-Item -LiteralPath `$updateScriptPath -Destination `$updateScriptOldPath -Force
            Write-SwapLog 'Backed up current .bat script version'
        }

        Move-Item -LiteralPath `$updateScriptNewPath -Destination `$updateScriptPath -Force
        Write-SwapLog 'Installed new .bat script version successfully'

        if (Test-Path -LiteralPath `$updateScriptOldPath) {
            Remove-Item -LiteralPath `$updateScriptOldPath -Force -ErrorAction SilentlyContinue
            Write-SwapLog 'Cleaned up old .bat script backup'
        }
    } else {
        Write-SwapLog 'No pending update found for .bat script'
    }

    # Handle PowerShell script
    if (Test-Path -LiteralPath `$updateScriptPsNewPath) {
        Write-SwapLog 'Found pending update for .ps1 script'

        if (Test-Path -LiteralPath `$updateScriptPsOldPath) {
            Remove-Item -LiteralPath `$updateScriptPsOldPath -Force
            Write-SwapLog 'Removed old backup for .ps1 script'
        }

        if (Test-Path -LiteralPath `$updateScriptPsPath) {
            Move-Item -LiteralPath `$updateScriptPsPath -Destination `$updateScriptPsOldPath -Force
            Write-SwapLog 'Backed up current .ps1 script version'
        }

        Move-Item -LiteralPath `$updateScriptPsNewPath -Destination `$updateScriptPsPath -Force
        Write-SwapLog 'Installed new .ps1 script version successfully'

        if (Test-Path -LiteralPath `$updateScriptPsOldPath) {
            Remove-Item -LiteralPath `$updateScriptPsOldPath -Force -ErrorAction SilentlyContinue
            Write-SwapLog 'Cleaned up old .ps1 script backup'
        }
    } else {
        Write-SwapLog 'No pending update found for .ps1 script'
    }
}
catch {
    Write-SwapLog "ERROR: Failed to swap files: `$(`$_.Exception.Message)"
    # Attempt rollback if current went missing but backup exists
    try {
        if (-not (Test-Path -LiteralPath `$updateScriptPath) -and (Test-Path -LiteralPath `$updateScriptOldPath)) {
            Move-Item -LiteralPath `$updateScriptOldPath -Destination `$updateScriptPath -Force
            Write-SwapLog 'Rolled back .bat script to previous version'
        }
        if (-not (Test-Path -LiteralPath `$updateScriptPsPath) -and (Test-Path -LiteralPath `$updateScriptPsOldPath)) {
            Move-Item -LiteralPath `$updateScriptPsOldPath -Destination `$updateScriptPsPath -Force
            Write-SwapLog 'Rolled back .ps1 script to previous version'
        }
    } catch {
        Write-SwapLog "ERROR: Rollback failed: `$(`$_.Exception.Message)"
    }
}
finally {
    # Remove the scheduled task if present
    try {
        if (Get-ScheduledTask -TaskName 'AdorsysUpdateSwap' -ErrorAction SilentlyContinue) {
            Unregister-ScheduledTask -TaskName 'AdorsysUpdateSwap' -Confirm:`$false -ErrorAction SilentlyContinue
            Write-SwapLog 'Update swap task completed and removed'
        } else {
            Write-SwapLog 'Scheduled task not found (nothing to remove)'
        }
    } catch {
        Write-SwapLog "ERROR: Failed to remove task: `$(`$_.Exception.Message)"
    }
}
"@

    try {
        # Check if task already exists and remove it
        $ExistingTask = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
        if ($ExistingTask) {
            Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction Stop
        }

        # Create the swap script file
        $SwapScriptDir = Split-Path -Path $SwapScriptPath -Parent
        if (-not (Test-Path $SwapScriptDir)) {
            New-Item -Path $SwapScriptDir -ItemType Directory -Force | Out-Null
        }
        Set-Content -Path $SwapScriptPath -Value $SwapScript -Force

        # Create the action to run the script
        $Action = New-ScheduledTaskAction -Execute "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe" -Argument "-ExecutionPolicy Bypass -WindowStyle Hidden -File `"$SwapScriptPath`""

        # Create a trigger that runs at logon
        $Trigger = New-ScheduledTaskTrigger -AtLogOn

        # Set to run with highest privileges using Administrators group
        $Principal = New-ScheduledTaskPrincipal -GroupId "BUILTIN\Administrators" -RunLevel Highest

        # Create settings
        $Settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable

        # Register the task
        Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $Trigger -Principal $Principal -Settings $Settings -Force | Out-Null

        InfoMessage "Scheduled task '$TaskName' created successfully"
        InfoMessage "The new version will be installed on next logon"
    } catch {
        ErrorMessage "Failed to create scheduled task: $($_.Exception.Message)"
    }
} else {
    InfoMessage "adorsys-update.bat is not running. Downloading directly..."
    Download-File -Url $UPDATE_SCRIPT_URL -OutputPath $UPDATE_SCRIPT_PATH
    InfoMessage "adorsys-update.ps1 is not running. Downloading directly..."
    Download-File -Url $UPDATE_SCRIPT_PS_URL -OutputPath $UPDATE_SCRIPT_PS_PATH
}

PrintStep 6 "Validating installation and configuration..."
Validate-Installation

SuccessMessage "Installation completed successfully!"