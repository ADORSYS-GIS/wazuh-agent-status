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

$APP_VERSION = if ($null -ne $env:APP_VERSION) { $env:APP_VERSION } else { "0.3.3" }

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

$UPDATE_SCRIPT_URL = "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/main/scripts/adorsys-update.ps1"
$UPDATE_SCRIPT_PATH = "${env:ProgramFiles(x86)}\ossec-agent\active-response\bin\adorsys-update.ps1"

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

PrintStep 1 "Downloading binaries..."
Download-File -Url $ServerURL -OutputPath "$BIN_DIR\$SERVER_NAME.exe"
Download-File -Url $ClientURL -OutputPath "$BIN_DIR\$CLIENT_NAME.exe"

# Configure server as a Windows service
PrintStep 2 "Configuring server service..."
Create-Service -ServiceName $SERVER_NAME -ExecutablePath $SERVER_EXE -DisplayName "Wazuh Agent Status Server" -Description "Wazuh Agent Status monitoring server."

# Add client to Windows startup
PrintStep 3 "Configuring client startup..."
Create-StartupShortcut -ShortcutName $CLIENT_NAME -ExecutablePath $CLIENT_EXE

# Download adorsys-update.ps1
PrintStep 4 "Downloading adorsys-update.ps1..."
Download-File -Url $UPDATE_SCRIPT_URL -OutputPath $UPDATE_SCRIPT_PATH

# Update WazuhManager default value in the downloaded adorsys-update.ps1
PrintStep 5 "Configuring WAZUH_MANAGER in adorsys-update.ps1..."
$UPDATE_SCRIPT_CONTENT = Get-Content $UPDATE_SCRIPT_PATH
$UPDATE_SCRIPT_CONTENT = $UPDATE_SCRIPT_CONTENT -replace '(\[string\]\$WazuhManager = ")[^"]*(")', "`$1$WAZUH_MANAGER`$2"
Set-Content -Path $UPDATE_SCRIPT_PATH -Value $UPDATE_SCRIPT_CONTENT

SuccessMessage "Installation completed successfully."