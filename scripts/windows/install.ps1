# Set strict mode for error handling
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Configuration
$APP_VERSION = if ($null -ne $env:APP_VERSION) { $env:APP_VERSION } else { "0.4.2.rc1" }

# Default Variables
$WAZUH_MANAGER = if ($null -ne $env:WAZUH_MANAGER) { $env:WAZUH_MANAGER } else { "wazuh.example.com" }
$SERVER_NAME = if ($null -ne $env:SERVER_NAME) { $env:SERVER_NAME } else { "wazuh-agent-status" }
$CLIENT_NAME = if ($null -ne $env:CLIENT_NAME) { $env:CLIENT_NAME } else { "wazuh-agent-status-client" }
$APP_VERSION = if ($env:APP_VERSION) { $env:APP_VERSION } else { "0.5.0-rc.1" }
$WAS_VERSION = $APP_VERSION

$WAZUH_AGENT_STATUS_REPO_REF = if ($null -ne $env:WAZUH_AGENT_STATUS_REPO_REF) { $env:WAZUH_AGENT_STATUS_REPO_REF } else { "main" }
$WAZUH_AGENT_STATUS_REPO_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$WAZUH_AGENT_STATUS_REPO_REF"

$TMP_DIR = Join-Path $env:TEMP "wazuh-agent-status-install"
if (-not (Test-Path $TMP_DIR)) {
    New-Item -Path $TMP_DIR -ItemType Directory | Out-Null
}

try {
    $ChecksumsURL = "$WAZUH_AGENT_STATUS_REPO_URL/checksums.sha256"
    $UtilsURL = "$WAZUH_AGENT_STATUS_REPO_URL/scripts/shared/utils.ps1"
    
    $global:ChecksumsPath = Join-Path $TMP_DIR "checksums.sha256"
    $UtilsPath = Join-Path $TMP_DIR "utils.ps1"

    Invoke-WebRequest -Uri $ChecksumsURL -OutFile $ChecksumsPath -ErrorAction Stop
    Invoke-WebRequest -Uri $UtilsURL -OutFile $UtilsPath -ErrorAction Stop

    # Verification function (bootstrap)
    function Get-FileChecksum-Bootstrap {
        param([string]$FilePath)
        return (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash.ToLower()
    }

    $ExpectedHash = (Select-String -Path $ChecksumsPath -Pattern "scripts/shared/utils.ps1").Line.Split(" ")[0]
    $ActualHash = Get-FileChecksum-Bootstrap -FilePath $UtilsPath

    if ([string]::IsNullOrWhiteSpace($ExpectedHash) -or ($ActualHash -ne $ExpectedHash.ToLower())) {
        Write-Error "Checksum verification failed for utils.ps1"
        Write-Error "Expected: $ExpectedHash"
        Write-Error "Got:      $ActualHash"
        exit 1
    }

    . $UtilsPath
}
catch {
    Write-Error "Failed to initialize utilities: $($_.Exception.Message)"
    exit 1
}

EnsureAdmin

# Default Variables
$WAZUH_MANAGER = if ($null -ne $env:WAZUH_MANAGER) { $env:WAZUH_MANAGER } else { "wazuh.example.com" }
$SERVER_NAME = if ($null -ne $env:SERVER_NAME) { $env:SERVER_NAME } else { "wazuh-agent-status" }
$CLIENT_NAME = if ($null -ne $env:CLIENT_NAME) { $env:CLIENT_NAME } else { "wazuh-agent-status-client" }

# Determine architecture
if (-not [Environment]::Is64BitOperatingSystem) {
    ErrorExit "Unsupported architecture: x86 (32-bit). Only amd64 (64-bit) is supported on Windows."
}
$ARCH = "amd64"
$BIN_DIR = "C:\Program Files\$SERVER_NAME"
$SERVER_EXE = "$BIN_DIR\$SERVER_NAME.exe"
$CLIENT_EXE = "$BIN_DIR\$CLIENT_NAME.exe"
$MIGRATION_MARKER = "C:\ProgramData\$SERVER_NAME\.migrated_from_go"

$BAT_UPDATE_SCRIPT_URL = "$WAZUH_AGENT_STATUS_REPO_URL/scripts/windows/adorsys-update.bat"
$BAT_UPDATE_SCRIPT_PATH = "${env:ProgramFiles(x86)}\ossec-agent\active-response\bin\adorsys-update.bat"

$PS_UPDATE_SCRIPT_URL = "$WAZUH_AGENT_STATUS_REPO_URL/scripts/windows/adorsys-update.ps1"
$PS_UPDATE_SCRIPT_PATH = "${env:ProgramFiles(x86)}\ossec-agent\active-response\bin\adorsys-update.ps1"

# Create necessary directories
Ensure-Directory -Path $BIN_DIR

# Download binaries
$BaseURL = "https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/download/v$WAS_VERSION"
$ServerURL = "$BaseURL/$SERVER_NAME-windows-$ARCH.exe"
$ClientURL = "$BaseURL/$CLIENT_NAME-windows-$ARCH.exe"
$BinChecksumsURL = "$BaseURL/checksums.sha256"
$global:ChecksumsURL = "$WAZUH_AGENT_STATUS_REPO_URL/checksums.sha256"

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

    # Validate adorsys-update batch script
    if (Test-Path -LiteralPath $BAT_UPDATE_SCRIPT_PATH) {
        SuccessMessage "adorsys-update batch script exists: $BAT_UPDATE_SCRIPT_PATH."
    } else {
        ErrorExit "adorsys-update batch script is missing: $BAT_UPDATE_SCRIPT_PATH."
    }

    # Validate adorsys-update PowerShell script
    if (Test-Path -LiteralPath $PS_UPDATE_SCRIPT_PATH) {
        SuccessMessage "adorsys-update PowerShell script exists: $PS_UPDATE_SCRIPT_PATH."
    } else {
        ErrorExit "adorsys-update PowerShell script is missing: $PS_UPDATE_SCRIPT_PATH."
    }

    SuccessMessage "Installation validation completed successfully."
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

PrintStep 1 "Checking migration status..."
if (Test-Path -LiteralPath $MIGRATION_MARKER) {
    InfoMessage "System already migrated from Go. Skipping legacy cleanup."
} else {
    PrintStep 1 "Stopping existing legacy Go processes..."
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

        Start-Sleep -Seconds 2
    } catch {
        WarnMessage "Error while stopping existing services/processes: $($_.Exception.Message)"
        WarnMessage "Continuing with installation..."
    }
}

PrintStep 2 "Downloading binaries..."
Download-And-VerifyFile -Url $ServerURL -Destination "$BIN_DIR\$SERVER_NAME.exe" -ChecksumPattern "$SERVER_NAME-windows-$ARCH.exe" -FileName "$SERVER_NAME" -ChecksumUrl $BinChecksumsURL
Download-And-VerifyFile -Url $ClientURL -Destination "$BIN_DIR\$CLIENT_NAME.exe" -ChecksumPattern "$CLIENT_NAME-windows-$ARCH.exe" -FileName "$CLIENT_NAME" -ChecksumUrl $BinChecksumsURL

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
    $batUpdateScriptNewPath = "$BAT_UPDATE_SCRIPT_PATH.new"
    Download-And-VerifyFile -Url $BAT_UPDATE_SCRIPT_URL -Destination $batUpdateScriptNewPath -ChecksumPattern "scripts/windows/adorsys-update.bat" -FileName "adorsys-update.bat"
    InfoMessage "New version downloaded to: $batUpdateScriptNewPath"
    InfoMessage "Creating scheduled task to replace script on next reboot..."

    # Also download PowerShell script
    $psUpdateScriptNewPath = "$PS_UPDATE_SCRIPT_PATH.new"
    Download-And-VerifyFile -Url $PS_UPDATE_SCRIPT_URL -Destination $psUpdateScriptNewPath -ChecksumPattern "scripts/windows/adorsys-update.ps1" -FileName "adorsys-update.ps1"
    InfoMessage "PowerShell version downloaded to: $psUpdateScriptNewPath"

    # Create a scheduled task to replace the script after logon
    $TaskName = "AdorsysUpdateSwap"
    $SwapScriptPath = "C:\ProgramData\ossec-agent\Run-UpdateSwap.ps1"
    $SwapScript = @"
#Requires -Version 5.1
`$ErrorActionPreference = 'Stop'

`$batUpdateScriptPath       = '$BAT_UPDATE_SCRIPT_PATH'
`$batUpdateScriptNewPath    = '$BAT_UPDATE_SCRIPT_PATH.new'
`$batUpdateScriptOldPath    = '$BAT_UPDATE_SCRIPT_PATH.old'
`$psUpdateScriptPath        = '$PS_UPDATE_SCRIPT_PATH'
`$psUpdateScriptNewPath     = '$PS_UPDATE_SCRIPT_PATH.new'
`$psUpdateScriptOldPath     = '$PS_UPDATE_SCRIPT_PATH.old'
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
    if (Test-Path -LiteralPath `$batUpdateScriptNewPath) {
        Write-SwapLog 'Found pending update for .bat script'

        if (Test-Path -LiteralPath `$batUpdateScriptOldPath) {
            Remove-Item -LiteralPath `$batUpdateScriptOldPath -Force
            Write-SwapLog 'Removed old backup for .bat script'
        }

        if (Test-Path -LiteralPath `$batUpdateScriptPath) {
            Move-Item -LiteralPath `$batUpdateScriptPath -Destination `$batUpdateScriptOldPath -Force
            Write-SwapLog 'Backed up current .bat script version'
        }

        Move-Item -LiteralPath `$batUpdateScriptNewPath -Destination `$batUpdateScriptPath -Force
        Write-SwapLog 'Installed new .bat script version successfully'

        if (Test-Path -LiteralPath `$batUpdateScriptOldPath) {
            Remove-Item -LiteralPath `$batUpdateScriptOldPath -Force -ErrorAction SilentlyContinue
            Write-SwapLog 'Cleaned up old .bat script backup'
        }
    } else {
        Write-SwapLog 'No pending update found for .bat script'
    }

    # Handle PowerShell script
    if (Test-Path -LiteralPath `$psUpdateScriptNewPath) {
        Write-SwapLog 'Found pending update for .ps1 script'

        if (Test-Path -LiteralPath `$psUpdateScriptOldPath) {
            Remove-Item -LiteralPath `$psUpdateScriptOldPath -Force
            Write-SwapLog 'Removed old backup for .ps1 script'
        }

        if (Test-Path -LiteralPath `$psUpdateScriptOldPath) {
            Move-Item -LiteralPath `$psUpdateScriptOldPath -Destination `$psUpdateScriptOldPath -Force
            Write-SwapLog 'Backed up current .ps1 script version'
        }

        Move-Item -LiteralPath `$psUpdateScriptNewPath -Destination `$psUpdateScriptOldPath -Force
        Write-SwapLog 'Installed new .ps1 script version successfully'

        if (Test-Path -LiteralPath `$psUpdateScriptOldPath) {
            Remove-Item -LiteralPath `$psUpdateScriptOldPath -Force -ErrorAction SilentlyContinue
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
        if (-not (Test-Path -LiteralPath `$batUpdateScriptPath) -and (Test-Path -LiteralPath `$batUpdateScriptOldPath)) {
            Move-Item -LiteralPath `$batUpdateScriptOldPath -Destination `$batUpdateScriptPath -Force
            Write-SwapLog 'Rolled back .bat script to previous version'
        }
        if (-not (Test-Path -LiteralPath `$psUpdateScriptOldPath) -and (Test-Path -LiteralPath `$psUpdateScriptOldPath)) {
            Move-Item -LiteralPath `$psUpdateScriptOldPath -Destination `$psUpdateScriptOldPath -Force
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
    Download-And-VerifyFile -Url $BAT_UPDATE_SCRIPT_URL -Destination $BAT_UPDATE_SCRIPT_PATH -FileName "adorsys-update.bat" -ChecksumPattern "scripts/windows/adorsys-update.bat" 
    InfoMessage "adorsys-update.ps1 is not running. Downloading directly..."
    Download-And-VerifyFile -Url $PS_UPDATE_SCRIPT_URL -Destination $PS_UPDATE_SCRIPT_PATH -FileName "adorsys-update.ps1" -ChecksumPattern "scripts/windows/adorsys-update.ps1"
}

# Create migration marker
$MarkerDir = Split-Path -Path $MIGRATION_MARKER -Parent
if (-not (Test-Path $MarkerDir)) { New-Item -Path $MarkerDir -ItemType Directory -Force | Out-Null }
if (-not (Test-Path $MIGRATION_MARKER)) { New-Item -Path $MIGRATION_MARKER -ItemType File -Force | Out-Null }

PrintStep 6 "Validating installation and configuration..."
Validate-Installation

SuccessMessage "Installation completed successfully!"