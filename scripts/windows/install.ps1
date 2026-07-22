# Set strict mode for error handling
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Configuration

$APP_VERSION = if ($env:APP_VERSION) { $env:APP_VERSION } else { "0.5.1" }
$WAZUH_MANAGER = if ($env:WAZUH_MANAGER) { $env:WAZUH_MANAGER } else { "wazuh.example.com" }
$SERVER_NAME = if ($env:SERVER_NAME) { $env:SERVER_NAME } else { "wazuh-agent-status" }
$CLIENT_NAME = if ($env:CLIENT_NAME) { $env:CLIENT_NAME } else { "wazuh-agent-status-client" }
$REPO_REF = if ($env:WAZUH_AGENT_STATUS_REPO_REF) { $env:WAZUH_AGENT_STATUS_REPO_REF } else { "user-main" }
$REPO_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$REPO_REF"
$TMP = Join-Path $env:TEMP "wazuh-agent-status-install"; if (-not (Test-Path $TMP)) { mkdir $TMP | Out-Null }

try {
    $global:ChecksumsPath = Join-Path $TMP "checksums.sha256"; $U = Join-Path $TMP "utils.ps1"
    Invoke-WebRequest "$REPO_URL/checksums.sha256" -OutFile $global:ChecksumsPath
    Invoke-WebRequest "$REPO_URL/scripts/shared/utils.ps1" -OutFile $U
    if ((Get-FileHash $U -Alg SHA256).Hash.ToLower() -ne (Select-String -Path $global:ChecksumsPath -Pattern "scripts/shared/utils.ps1").Line.Split(" ")[0].ToLower()) { throw }
    . $U
} catch { Write-Error "Bootstrap failed: $($_.Exception.Message)"; exit 1 }

EnsureWindows
EnsureAdmin

# Determine architecture
if (-not [Environment]::Is64BitOperatingSystem) {
    ErrorExit "Unsupported architecture: x86 (32-bit). Only amd64 (64-bit) is supported on Windows."
}
$ARCH = "amd64"
$BIN_DIR = "C:\Program Files\$SERVER_NAME"
$SERVER_EXE = "$BIN_DIR\$SERVER_NAME.exe"
$CLIENT_EXE = "$BIN_DIR\$CLIENT_NAME.exe"
$MIGRATION_MARKER = "C:\ProgramData\$SERVER_NAME\.migrated_from_go"

$BAT_UPDATE_SCRIPT_URL = "$REPO_URL/scripts/windows/adorsys-update.bat"
$BAT_UPDATE_SCRIPT_PATH = "${env:ProgramFiles(x86)}\ossec-agent\active-response\bin\adorsys-update.bat"

$PS_UPDATE_SCRIPT_URL = "$REPO_URL/scripts/windows/adorsys-update.ps1"
$PS_UPDATE_SCRIPT_PATH = "${env:ProgramFiles(x86)}\ossec-agent\active-response\bin\adorsys-update.ps1"

# Create necessary directories
Ensure-Directory -Path $BIN_DIR

# Download binaries
$BaseURL = if ($null -ne $env:BASE_URL) { $env:BASE_URL } else { "https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/download/v$APP_VERSION" }
$ServerURL = "$BaseURL/$SERVER_NAME-windows-$ARCH.exe"
$ClientURL = "$BaseURL/$CLIENT_NAME-windows-$ARCH.exe"
$BinChecksumsURL = "$BaseURL/checksums.sha256"
$global:ChecksumsURL = "$REPO_URL/checksums.sha256"

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
    $startupShortcutPath = [System.IO.Path]::Combine($env:ProgramData, "Microsoft\Windows\Start Menu\Programs\Startup", "$CLIENT_NAME.lnk")
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
    $ServiceExists = Get-CimInstance -ClassName Win32_Service -Filter "Name='$ServiceName'" -ErrorAction SilentlyContinue

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
    $ShortcutPath = [System.IO.Path]::Combine($env:ProgramData, "Microsoft\Windows\Start Menu\Programs\Startup", "$ShortcutName.lnk")
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = $ExecutablePath
    $Shortcut.Save()
    InfoMessage "Startup shortcut created: $ShortcutPath."
}

function Register-Uninstaller {
    param(
        [string]$DisplayName,
        [string]$DisplayVersion,
        [string]$Publisher,
        [string]$DisplayIcon,
        [string]$UninstallString
    )
    $RegistryPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\WazuhAgentStatus"
    if (-not (Test-Path $RegistryPath)) {
        New-Item -Path $RegistryPath -Force | Out-Null
    }
    Set-ItemProperty -Path $RegistryPath -Name "DisplayName" -Value $DisplayName
    Set-ItemProperty -Path $RegistryPath -Name "DisplayVersion" -Value $DisplayVersion
    Set-ItemProperty -Path $RegistryPath -Name "Publisher" -Value $Publisher
    Set-ItemProperty -Path $RegistryPath -Name "DisplayIcon" -Value $DisplayIcon
    Set-ItemProperty -Path $RegistryPath -Name "UninstallString" -Value $UninstallString
    Set-ItemProperty -Path $RegistryPath -Name "NoModify" -Value 1 -Type DWord
    Set-ItemProperty -Path $RegistryPath -Name "NoRepair" -Value 1 -Type DWord
    InfoMessage "Registered application in Windows Installed Apps."
}

function Create-StartMenuShortcut {
    param(
        [string]$ShortcutName,
        [string]$ExecutablePath
    )
    $StartMenuPath = [System.IO.Path]::Combine($env:ProgramData, "Microsoft\Windows\Start Menu\Programs", "$ShortcutName.lnk")
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($StartMenuPath)
    $Shortcut.TargetPath = $ExecutablePath
    $Shortcut.Save()
    InfoMessage "Start Menu shortcut created: $StartMenuPath."
}

PrintStep 1 "Checking migration status and stopping existing processes..."

if (Test-Path -LiteralPath $MIGRATION_MARKER) {
    InfoMessage "System already migrated from Go."
} else {
    InfoMessage "System not migrated from Go yet."
}

InfoMessage "Ensuring all running instances are stopped before downloading new binaries..."
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

PrintStep 2 "Downloading binaries..."
Download-And-VerifyFile -Url $ServerURL -Destination "$BIN_DIR\$SERVER_NAME.exe" -ChecksumPattern "$SERVER_NAME-windows-$ARCH.exe" -FileName "$SERVER_NAME" -ChecksumUrl $BinChecksumsURL
Download-And-VerifyFile -Url $ClientURL -Destination "$BIN_DIR\$CLIENT_NAME.exe" -ChecksumPattern "$CLIENT_NAME-windows-$ARCH.exe" -FileName "$CLIENT_NAME" -ChecksumUrl $BinChecksumsURL

# Configure server as a Windows service
PrintStep 3 "Configuring server service..."
Create-Service -ServiceName $SERVER_NAME -ExecutablePath $SERVER_EXE -DisplayName "Wazuh Agent Status Server" -Description "Wazuh Agent Status monitoring server."

# Add client to Windows startup, Start Menu, and Installed Apps
PrintStep 4 "Configuring client startup and registration..."
Create-StartupShortcut -ShortcutName $CLIENT_NAME -ExecutablePath $CLIENT_EXE
Create-StartMenuShortcut -ShortcutName "Wazuh Agent Status" -ExecutablePath $CLIENT_EXE
$UninstallCmd = "powershell.exe -WindowStyle Hidden -ExecutionPolicy Bypass -Command `"Invoke-RestMethod -Uri '$REPO_URL/scripts/windows/uninstall.ps1' | Invoke-Expression`""
Register-Uninstaller -DisplayName "Wazuh Agent Status" -DisplayVersion $APP_VERSION -Publisher "ADORSYS" -DisplayIcon $CLIENT_EXE -UninstallString $UninstallCmd

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
    function Swap-File {
        param([string]`$Path, [string]`$NewPath, [string]`$OldPath, [string]`$Ext)
        if (Test-Path -LiteralPath `$NewPath) {
            Write-SwapLog "Found pending update for `$Ext script"
            if (Test-Path -LiteralPath `$OldPath) {
                Remove-Item -LiteralPath `$OldPath -Force
                Write-SwapLog "Removed old backup for `$Ext script"
            }
            if (Test-Path -LiteralPath `$Path) {
                Move-Item -LiteralPath `$Path -Destination `$OldPath -Force
                Write-SwapLog "Backed up current `$Ext script version"
            }
            Move-Item -LiteralPath `$NewPath -Destination `$Path -Force
            Write-SwapLog "Installed new `$Ext script version successfully"
            if (Test-Path -LiteralPath `$OldPath) {
                Remove-Item -LiteralPath `$OldPath -Force -ErrorAction SilentlyContinue
                Write-SwapLog "Cleaned up old `$Ext script backup"
            }
        } else {
            Write-SwapLog "No pending update found for `$Ext script"
        }
    }

    Swap-File -Path `$batUpdateScriptPath -NewPath `$batUpdateScriptNewPath -OldPath `$batUpdateScriptOldPath -Ext ".bat"
    Swap-File -Path `$psUpdateScriptPath -NewPath `$psUpdateScriptNewPath -OldPath `$psUpdateScriptOldPath -Ext ".ps1"
}
catch {
    Write-SwapLog "ERROR: Failed to swap files: `$(`$_.Exception.Message)"
    # Attempt rollback if current went missing but backup exists
    try {
        if (-not (Test-Path -LiteralPath `$batUpdateScriptPath) -and (Test-Path -LiteralPath `$batUpdateScriptOldPath)) {
            Move-Item -LiteralPath `$batUpdateScriptOldPath -Destination `$batUpdateScriptPath -Force
            Write-SwapLog 'Rolled back .bat script to previous version'
        }
        if (-not (Test-Path -LiteralPath `$psUpdateScriptPath) -and (Test-Path -LiteralPath `$psUpdateScriptOldPath)) {
            Move-Item -LiteralPath `$psUpdateScriptOldPath -Destination `$psUpdateScriptPath -Force
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
