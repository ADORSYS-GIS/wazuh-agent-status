# Define variables used in the installation script
$APP_NAME = if ($env:APP_NAME -ne $null) { $env:APP_NAME } else { "wazuh-agent-status" }
$DEFAULT_WOPS_VERSION = "0.1.2"
$WOPS_VERSION = if ($env:WOPS_VERSION -ne $null) { $env:WOPS_VERSION } else { $DEFAULT_WOPS_VERSION }
$BIN_DIR_AMD64 = "C:\Program Files (x86)\ossec-agent"
$BIN_DIR_AMD32 = "C:\Program Files\ossec-agent"
$BIN_DIR = if ([Environment]::Is64BitOperatingSystem) { $BIN_DIR_AMD64 } else { $BIN_DIR_AMD32 }
$APP_PATH = "$BIN_DIR\$APP_NAME.exe"

# Function for logging
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

# Ensure script runs with administrative privileges
function EnsureAdmin {
    if (-Not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
        ErrorMessage "This script requires administrative privileges. Please run it as Administrator."
        exit 1
    }
}

# Function to uninstall application and clean up
function Uninstall-WazuhAgentStatus {
    InfoMessage "Starting uninstallation of $APP_NAME."

    # Step 1: Remove binary
    if (Test-Path -Path $APP_PATH) {
        InfoMessage "Removing application binary at $APP_PATH..."
        Remove-Item -Path $APP_PATH -Force -ErrorAction SilentlyContinue
        if (-Not (Test-Path -Path $APP_PATH)) {
            SuccessMessage "$APP_NAME binary removed."
        } else {
            ErrorMessage "Failed to remove $APP_NAME binary."
        }
    } else {
        WarnMessage "$APP_NAME binary not found at $APP_PATH. Skipping removal."
    }

    # Step 2: Remove application directory if empty
    if (Test-Path -Path $BIN_DIR) {
        InfoMessage "Removing application directory $BIN_DIR if empty..."
        try {
            Remove-Item -Path $BIN_DIR -Recurse -Force -ErrorAction SilentlyContinue
            if (-Not (Test-Path -Path $BIN_DIR)) {
                SuccessMessage "$BIN_DIR removed successfully."
            } else {
                WarnMessage "$BIN_DIR is not empty or could not be removed."
            }
        } catch {
            ErrorMessage "Failed to remove $BIN_DIR. Error: $_"
        }
    } else {
        WarnMessage "Application directory $BIN_DIR not found. Skipping removal."
    }

    # Step 3: Cleanup temporary files (if applicable)
    $TEMP_FILE_PATTERN = "*wazuh-agent-status*.exe"
    $TEMP_FILES = Get-ChildItem -Path $env:TEMP -Filter $TEMP_FILE_PATTERN -Recurse -ErrorAction SilentlyContinue
    if ($TEMP_FILES -ne $null) {
        InfoMessage "Removing temporary files related to $APP_NAME..."
        foreach ($file in $TEMP_FILES) {
            try {
                Remove-Item -Path $file.FullName -Force -ErrorAction SilentlyContinue
                SuccessMessage "Removed temporary file: $($file.FullName)"
            } catch {
                ErrorMessage "Failed to remove temporary file: $($file.FullName). Error: $_"
            }
        }
    } else {
        InfoMessage "No temporary files related to $APP_NAME found."
    }

    SuccessMessage "Uninstallation of $APP_NAME completed."
}

# Ensure admin privileges
EnsureAdmin

# Run the uninstallation
Uninstall-WazuhAgentStatus
