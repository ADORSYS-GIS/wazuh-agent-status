#!/bin/sh

# Set shell options based on shell type
if [ -n "$BASH_VERSION" ]; then
    set -euo pipefail
else
    set -eu
fi

# Default log level and application details
APP_NAME=${APP_NAME:-"wazuh-agent-status"}
WOPS_VERSION=${WOPS_VERSION:-"0.1.2"}
WAZUH_USER=${WAZUH_USER:-"root"}
SERVICE_FILE=${SERVICE_FILE:-"/etc/systemd/system/$APP_NAME.service"}
PROFILE_RC=${PROFILE_RC:-"$HOME/.profile"}

# Define text formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[1;34m'
BOLD='\033[1m'
NORMAL='\033[0m'

# Function for logging with timestamp
log() {
    local LEVEL="$1"
    shift
    local MESSAGE="$*"
    local TIMESTAMP
    TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
    echo -e "${TIMESTAMP} ${LEVEL} ${MESSAGE}"
}

# Logging helpers
info_message() {
    log "${BLUE}${BOLD}[INFO]${NORMAL}" "$*"
}

warn_message() {
    log "${YELLOW}${BOLD}[WARNING]${NORMAL}" "$*"
}

error_message() {
    log "${RED}${BOLD}[ERROR]${NORMAL}" "$*"
}

success_message() {
    log "${GREEN}${BOLD}[SUCCESS]${NORMAL}" "$*"
}

print_step() {
    log "${BLUE}${BOLD}[STEP]${NORMAL}" "$1: $2"
}

# Exit script with an error message
error_exit() {
    error_message "$1"
    exit 1
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Ensure root privileges, either directly or through sudo
maybe_sudo() {
    if [ "$(id -u)" -ne 0 ]; then
        if command_exists sudo; then
            sudo "$@"
        else
            error_message "This script requires root privileges. Please run with sudo or as root."
            exit 1
        fi
    else
        "$@"
    fi
}

# Function to create a launchd plist file for macOS
create_launchd_plist() {
    local PLIST_FILE="/Library/LaunchDaemons/com.example.$APP_NAME.plist"

    if [ -f "$PLIST_FILE" ]; then
        info_message "LaunchDaemon plist $PLIST_FILE already exists. Removing..."
        sudo rm "$PLIST_FILE"
    fi

    info_message "Creating LaunchDaemon plist at $PLIST_FILE..."

    sudo tee "$PLIST_FILE" > /dev/null <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.example.$APP_NAME</string>
    <key>ProgramArguments</key>
    <array>
        <string>$BIN_DIR/$APP_NAME</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/$APP_NAME.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/$APP_NAME.err</string>
    <key>UserName</key>
    <string>$WAZUH_USER</string>
</dict>
</plist>
EOF

    info_message "LaunchDaemon plist created."
}

# Function to load and start the launchd service (macOS)
load_and_start_launchd_service() {
    local PLIST_FILE="/Library/LaunchDaemons/com.example.$APP_NAME.plist"

    info_message "Loading and starting LaunchDaemon service..."
    sudo launchctl load -w "$PLIST_FILE"
    sudo launchctl start "com.example.$APP_NAME"
}

# Determine the OS and architecture
case "$(uname)" in
    "Darwin") 
        OS="darwin"
        BIN_DIR="/usr/local/bin"
        ;;
    *) 
        error_exit "This script is intended for macOS only."
        ;;
esac

ARCH=$(uname -m)
case "$ARCH" in
    "x86_64") ARCH="amd64" ;;
    "arm64") ARCH="arm64" ;;
    *) error_exit "Unsupported architecture: $ARCH" ;;
esac

# Construct binary name and URL for download
BIN_NAME="$APP_NAME-${OS}-${ARCH}"
BASE_URL="https://github.com/ADORSYS-GIS/$APP_NAME/releases/download/v$WOPS_VERSION"
URL="$BASE_URL/$BIN_NAME"

echo "Download URL: $URL"

# Create a temporary directory and ensure it is cleaned up
TEMP_DIR=$(mktemp -d) || error_exit "Failed to create temporary directory"

trap 'rm -rf "$TEMP_DIR"' EXIT

# Step 1: Download the binary file
print_step 1 "Downloading $BIN_NAME from $URL..."
curl -SL --progress-bar -o "$TEMP_DIR/$BIN_NAME" "$URL" || error_exit "Failed to download $BIN_NAME"

# Step 2: Install the binary
print_step 2 "Installing binary to $BIN_DIR..."
maybe_sudo mv "$TEMP_DIR/$BIN_NAME" "$BIN_DIR/$APP_NAME" || error_exit "Failed to move binary to $BIN_DIR"
maybe_sudo chmod 755 "$BIN_DIR/$APP_NAME" || error_exit "Failed to set executable permissions on the binary"

# Step 3: Set up and start the service
print_step 3 "Starting service creation process..."
create_launchd_plist
load_and_start_launchd_service

info_message "Service creation and setup complete."

success_message "Installation and configuration complete! The $APP_NAME service should now be running."
info_message "You can check its status using: sudo launchctl list | grep $APP_NAME"
info_message "Logs can be found at /tmp/$APP_NAME.log and /tmp/$APP_NAME.err"