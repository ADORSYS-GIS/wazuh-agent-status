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

# macOS-specific service file location
MAC_SERVICE_FILE="$HOME/Library/LaunchAgents/$APP_NAME.plist"

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

# Function to reload systemd and enable the service (Linux-specific)
remove_service() {
    info_message "Stopping $APP_NAME service..."
    sudo systemctl stop $APP_NAME
    
    info_message "Deleting $SERVICE_FILE file..."
    sudo rm $SERVICE_FILE
    
    info_message "Reloading systemd daemon..."
    sudo systemctl daemon-reload
}

# macOS: Function to create launchd plist file
create_mac_service_file() {
    if [ -f "$MAC_SERVICE_FILE" ]; then
        info_message "Launchd service file $MAC_SERVICE_FILE already exists. Deleting..."
        remove_mac_service
        info_message "Old version of service file deleted successfully"
    fi

    info_message "Creating launchd plist file at $MAC_SERVICE_FILE..."

    cat > "$MAC_SERVICE_FILE" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
    <dict>
        <key>Label</key>
        <string>$APP_NAME</string>
        <key>ProgramArguments</key>
        <array>
            <string>$BIN_DIR/$APP_NAME</string>
        </array>
        <key>RunAtLoad</key>
        <true/>
        <key>KeepAlive</key>
        <true/>
        <key>WorkingDirectory</key>
        <string>/</string>
        <key>StandardErrorPath</key>
        <string>/tmp/$APP_NAME.err</string>
        <key>StandardOutPath</key>
        <string>/tmp/$APP_NAME.out</string>
    </dict>
</plist>
EOF

    info_message "Launchd plist file created."
}

# macOS: Function to remove launchd service
remove_mac_service() {
    info_message "Unloading $APP_NAME service..."
    launchctl bootout user/$(id -u) "$MAC_SERVICE_FILE"
    
    info_message "Deleting $MAC_SERVICE_FILE file..."
    rm "$MAC_SERVICE_FILE"
}

# macOS: Function to reload and enable the service
reload_and_enable_mac_service() {
    info_message "Loading $APP_NAME service with launchd..."
    launchctl bootout user/$(id -u) "$MAC_SERVICE_FILE" || true
    launchctl bootstrap user/$(id -u) "$MAC_SERVICE_FILE"

    info_message "Service $APP_NAME has been started. It will now start automatically on boot."
}

# Function to check if the binary exists
check_binary_exists() {
    if [ ! -f "$BIN_DIR" ]; then
        warn_message "Binary $BIN_DIR does not exist. Exiting."
        exit 1
    fi
}

# Determine the OS and architecture
case "$(uname)" in
    "Linux") OS="linux"; BIN_DIR="/usr/local/bin" ;;
    "Darwin") OS="darwin"; BIN_DIR="/usr/local/bin" ;;
    *) error_exit "Unsupported operating system: $(uname)" ;;
esac

ARCH=$(uname -m)
case "$ARCH" in
    "x86_64") ARCH="amd64" ;;
    "arm64"|"aarch64") ARCH="arm64" ;;
    *) error_exit "Unsupported architecture: $ARCH" ;;
esac

# Construct binary name and URL for download
BIN_NAME="$APP_NAME-${OS}-${ARCH}"
BASE_URL="https://github.com/ADORSYS-GIS/$APP_NAME/releases/download/v$WOPS_VERSION"
URL="$BASE_URL/$BIN_NAME"

echo $URL

# Create a temporary directory and ensure it is cleaned up
TEMP_DIR=$(mktemp -d) || error_exit "Failed to create temporary directory"
trap 'rm -rf "$TEMP_DIR"' EXIT

# Step 1: Download the binary file
print_step 1 "Downloading $BIN_NAME from $URL..."
curl -SL --progress-bar -o "$TEMP_DIR/$BIN_NAME" "$URL" || error_exit "Failed to download $BIN_NAME"

# Step 2: Install the binary
print_step 2 "Installing binary to $BIN_DIR..."
maybe_sudo mv "$TEMP_DIR/$BIN_NAME" "$BIN_DIR/$APP_NAME" || error_exit "Failed to move binary to $BIN_DIR"
maybe_sudo chmod 111 "$BIN_DIR/$APP_NAME" || error_exit "Failed to set executable permissions on the binary"

# Step 3: Run the binary as a service
print_step 3 "Starting service creation process..."
grant_display_access
if [ "$OS" = "darwin" ]; then
    create_mac_service_file
    reload_and_enable_mac_service
else
    create_service_file
    reload_and_enable_service
fi

info_message "Service creation and setup complete."

success_message "Installation and configuration complete! You can now use '$APP_NAME' from your terminal."
info_message "Run \n\n\t${GREEN}${BOLD}$APP_NAME ${NORMAL}\n\n to start configuring. If you don't have sudo on your machine, you can run the command without sudo."
