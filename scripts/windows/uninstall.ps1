# Set strict mode for error handling
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Configuration
$APP_VERSION = if ($env:APP_VERSION) { $env:APP_VERSION } else { "0.4.3" }
$WAS_VERSION = if ($env:INSTALL_PROFILE -eq "admin") { $APP_VERSION } else { "$APP_VERSION-user" }
$REPO_REF    = if ($env:WAZUH_AGENT_STATUS_REPO_REF) { $env:WAZUH_AGENT_STATUS_REPO_REF } else { "refs/tags/v$WAS_VERSION" }
$REPO_URL    = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$REPO_REF"
$TMP_DIR     = Join-Path $env:TEMP "wazuh-agent-status-install"; if (!(Test-Path $TMP_DIR)) { mkdir $TMP_DIR | Out-Null }

try {
    $global:ChecksumsPath = Join-Path $TMP_DIR "checksums.sha256"; $UtilsPath = Join-Path $TMP_DIR "utils.ps1"
    Invoke-WebRequest -Uri "$REPO_URL/checksums.sha256" -OutFile $global:ChecksumsPath -ErrorAction Stop
    Invoke-WebRequest -Uri "$REPO_URL/scripts/shared/utils.ps1" -OutFile $UtilsPath -ErrorAction Stop
    if ((Get-FileHash $UtilsPath -Algorithm SHA256).Hash.ToLower() -ne (Select-String $global:ChecksumsPath -Pattern "scripts/shared/utils.ps1").Line.Split(" ")[0].Trim().ToLower()) { throw "Checksum mismatch" }
    . $UtilsPath
} catch { Write-Error "Initialization failed: $_"; exit 1 }

EnsureAdmin

# Environment Variables with Defaults
$SERVER_NAME = if ($null -ne $env:SERVER_NAME) { $env:SERVER_NAME } else { "wazuh-agent-status" }
$CLIENT_NAME = if ($null -ne $env:CLIENT_NAME) { $env:CLIENT_NAME } else { "wazuh-agent-status-client" }
$BIN_DIR = "C:\Program Files\$SERVER_NAME"
$SERVER_EXE = "$BIN_DIR\$SERVER_NAME.exe"
$CLIENT_EXE = "$BIN_DIR\$CLIENT_NAME.exe"
$BAT_UPDATE_SCRIPT_PATH = "${env:ProgramFiles(x86)}\ossec-agent\active-response\bin\adorsys-update.bat"
$PS_UPDATE_SCRIPT_PATH = "${env:ProgramFiles(x86)}\ossec-agent\active-response\bin\adorsys-update.ps1"

function Remove-File {
    param (
        [Parameter(Mandatory = $true)]
        [string]$FilePath
    )
    InfoMessage "Removing '$FilePath'"
    try {
        if (Test-Path -Path $FilePath) {
            Remove-Item -Path $FilePath -Force -ErrorAction Stop
            InfoMessage "File '$FilePath' has been successfully removed."
        } else {
            WarnMessage "File '$FilePath' does not exist."
        }
    } catch {
        ErrorMessage "An error occurred while trying to remove the file: $_"
    }
}

function Remove-Service {
    param (
        [Parameter(Mandatory=$true)]
        [string]$ServiceName
    )

    # Check if the service exists
    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue

    if ($service) {
        # Stop the service if it's running
        if ($service.Status -eq 'Running') {

            Stop-Service -Name $ServiceName -Force
        }

        # Remove the service using sc.exe
        sc.exe delete $ServiceName | Out-Null

        InfoMessage "Service '$ServiceName' has been removed successfully."
    } else {
        WarnMessage "Service '$ServiceName' not found."
    }
}

function Remove-StartupShortcut {
    param (
        [Parameter(Mandatory = $true)]
        [string]$ShortcutName
    )


    # Check if the process is running
    $process = Get-Process -Name $ShortcutName -ErrorAction SilentlyContinue

    if ($process) {
        InfoMessage "Process '$ShortcutName' is running. Stopping it..."
        Stop-Process -Name $ShortcutName -Force
        InfoMessage "Process '$ShortcutName' has been stopped."
    }
    else {
        WarnMessage "Process '$ShortcutName' is not running. Skipping..."
    }
    # Define full path of the shortcut

    InfoMessage "Removing Shortcut '$ShortcutName' from Startup..."
    $ShortcutPath = [System.IO.Path]::Combine($env:APPDATA, "Microsoft\Windows\Start Menu\Programs\Startup", "$ShortcutName.lnk")

    # Check if the shortcut exists and remove it
    if (Test-Path $ShortcutPath) {
        Remove-Item -Path $ShortcutPath -Force
        InfoMessage "Shortcut '$ShortcutName' removed from Startup."
    } else {
        WarnMessage "Shortcut '$ShortcutName' not found in Startup."
    }
}

function Validate-Uninstallation {
    $ServerService = Get-Service -Name $SERVER_NAME -ErrorAction SilentlyContinue
    $ClientProcess = Get-Process -Name $CLIENT_NAME -ErrorAction SilentlyContinue
    $BatUpdateScript = Test-Path -LiteralPath $BAT_UPDATE_SCRIPT_PATH
    $PsUpdateScript = Test-Path -LiteralPath $PS_UPDATE_SCRIPT_PATH
    $ServerExe = Test-Path -LiteralPath $SERVER_EXE
    $ClientExe = Test-Path -LiteralPath $CLIENT_EXE
    $BinDirExists = Test-Path -LiteralPath $BIN_DIR

    if ($ServerService -eq $null) {
        SuccessMessage "Windows service is removed: $SERVER_NAME."
    }
    else {
        ErrorMessage "Windows service still exists: $SERVER_NAME (current status: $($ServerService.Status))."
    }

    if ($ClientProcess -eq $null) {
        SuccessMessage "Client process is not running: $CLIENT_NAME."
    }
    else {
        ErrorMessage "Client process is still running: $CLIENT_NAME (current status: $($ClientProcess.Status))."
    }

    if ($BatUpdateScript -eq $false) {
        SuccessMessage "adorsys-update batch script is removed: $BAT_UPDATE_SCRIPT_PATH."
    }
    else {
        ErrorMessage "adorsys-update batch script still exists: $BAT_UPDATE_SCRIPT_PATH."
    }

    if ($PsUpdateScript -eq $false) {
        SuccessMessage "adorsys-update PowerShell script is removed: $PS_UPDATE_SCRIPT_PATH."
    }
    else {
        ErrorMessage "adorsys-update PowerShell script still exists: $PS_UPDATE_SCRIPT_PATH."
    }

    if ($ServerExe -eq $false) {
        SuccessMessage "Server binary is removed: $SERVER_EXE."
    }
    else {
        ErrorMessage "Server binary still exists: $SERVER_EXE."
    }

    if ($ClientExe -eq $false) {
        SuccessMessage "Client binary is removed: $CLIENT_EXE."
    }
    else {
        ErrorMessage "Client binary still exists: $CLIENT_EXE."
    }

    if ($BinDirExists -eq $false) {
        SuccessMessage "Bin directory is removed: $BIN_DIR."
    }
    else {
        ErrorMessage "Bin directory still exists: $BIN_DIR."
    }
}

function Remove-Binaries {
    Remove-File $SERVER_EXE
    Remove-File $CLIENT_EXE
    Remove-File $BAT_UPDATE_SCRIPT_PATH
    Remove-File $PS_UPDATE_SCRIPT_PATH
    Remove-File $BIN_DIR
}

# Function to uninstall application and clean up
function Uninstall-WazuhAgentStatus {
    try {

        Remove-StartupShortcut -ShortcutName $CLIENT_NAME
        Remove-Service -ServiceName $SERVER_NAME

        Remove-Binaries
        Validate-Uninstallation
        SuccessMessage "Wazuh Agent Status uninstalled successfully"
    }
    catch {
        ErrorMessage "Wazuh Agent Status Uninstall Failed: $($_.Exception.Message)"
    }
}

Uninstall-WazuhAgentStatus
