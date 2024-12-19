# Ensure the script is running as administrator
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "This script must be run as Administrator."
    exit 1
}

# Default Variables
$SERVER_NAME = $env:SERVER_NAME -or "wazuh-agent-status"
$CLIENT_NAME = $env:CLIENT_NAME -or "wazuh-agent-status-client"
$WAZUH_USER = $env:WAZUH_USER -or "NT AUTHORITY\SYSTEM"
$PROFILE = $env:PROFILE -or "user"
$APP_VERSION = $env:APP_VERSION -or "0.2.1"

if ($PROFILE -eq "admin") {
    $WAS_VERSION = $APP_VERSION
} else {
    $WAS_VERSION = "$APP_VERSION-user"
}

$BIN_DIR = "C:\Program Files\$SERVER_NAME"
$SERVER_EXE = "$BIN_DIR\$SERVER_NAME.exe"
$CLIENT_EXE = "$BIN_DIR\$CLIENT_NAME.exe"

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
function InfoMessage {
    param ([string]$Message)
    Log "[INFO]" $Message
}

function WarnMessage {
    param ([string]$Message)
    Log "[WARNING]" $Message
}

function ErrorMessage {
    param ([string]$Message)
    Log "[ERROR]" $Message
}

function SuccessMessage {
    param ([string]$Message)
    Log "[SUCCESS]" $Message
}

function PrintStep {
    param (
        [int]$StepNumber,
        [string]$Message
    )
    Log "[STEP]" "Step ${StepNumber}: $Message"
}

# Exit script with an error message
function ErrorExit {
    param ([string]$Message)
    ErrorMessage $Message
    exit 1
}

# # Utility Functions
# function InfoMessage {
#     param(
#         [string]$Message,
#         [string]$Level = "INFO"
#     )
#     Write-Host "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') [$Level] $Message"
# }

function Download-File {
    param(
        [string]$Url,
        [string]$OutputPath
    )
    try {
        Invoke-WebRequest -Uri $Url -OutFile $OutputPath
        InfoMessage "Downloaded $OutputPath from $Url."
    } catch {
        ErrorExit "Failed to download $Url." "ERROR"
    }
}

function Create-Service {
    param(
        [string]$ServiceName,
        [string]$ExecutablePath,
        [string]$DisplayName = $null,
        [string]$Description = $null
    )
    if (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue) {
        InfoMessage "Service $ServiceName already exists. Updating..."
        Stop-Service -Name $ServiceName -Force
        Remove-Service -Name $ServiceName
    }

    InfoMessage "Creating service $ServiceName..."
    New-Service -Name $ServiceName -BinaryPathName "`"$ExecutablePath`"" -StartupType Automatic -Description $Description -DisplayName $DisplayName
    Start-Service -Name $ServiceName
    InfoMessage "Service $ServiceName created and started."
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
$ServerURL = "$BaseURL/$SERVER_NAME-windows-amd64.exe"
$ClientURL = "$BaseURL/$CLIENT_NAME-windows-amd64.exe"

PrintStep 1 "Downloading binaries..."
Download-File -Url $ServerURL -OutputPath "$BIN_DIR\$SERVER_NAME.exe"
Download-File -Url $ClientURL -OutputPath "$BIN_DIR\$CLIENT_NAME.exe"

# Configure server as a Windows service
PrintStep 2 "Configuring server service..."
Create-Service -ServiceName $SERVER_NAME -ExecutablePath $SERVER_EXE -DisplayName "Wazuh Agent Status Server" -Description "Wazuh Agent Status monitoring server."

# Add client to Windows startup
PrintStep 3 "Configuring client startup..."
Create-StartupShortcut -ShortcutName $CLIENT_NAME -ExecutablePath $CLIENT_EXE

SuccessMessage "Installation completed successfully." "SUCCESS"
