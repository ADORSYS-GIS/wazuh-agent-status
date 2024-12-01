#!/bin/sh

# Set shell options
set -eu

# Default variables
SERVER_NAME=${SERVER_NAME:-"wazuh-agent-status"}
CLIENT_NAME=${CLIENT_NAME:-"wazuh-agent-status-client"}
WOPS_VERSION=${WOPS_VERSION:-"0.2.1"}
WAZUH_USER=${WAZUH_USER:-"root"}
SERVER_LAUNCH_AGENT_FILE=${SERVER_LAUNCH_AGENT_FILE:-"$HOME/Library/LaunchAgents/com.$SERVER_NAME.plist"}
CLIENT_LAUNCH_AGENT_FILE=${CLIENT_LAUNCH_AGENT_FILE:-"$HOME/Library/LaunchAgents/com.$CLIENT_NAME.plist"}


# Text formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[1;34m'
BOLD='\033[1m'
NORMAL='\033[0m'

# Logging functions
log() {
    local LEVEL="$1"
    shift
    local MESSAGE="$*"
    local TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
    echo -e "${TIMESTAMP} ${LEVEL} ${MESSAGE}"
}

info_message() { log "${BLUE}${BOLD}[INFO]${NORMAL}" "$*"; }
warn_message() { log "${YELLOW}${BOLD}[WARNING]${NORMAL}" "$*"; }
error_message() { log "${RED}${BOLD}[ERROR]${NORMAL}" "$*"; }
success_message() { log "${GREEN}${BOLD}[SUCCESS]${NORMAL}" "$*"; }
print_step() { log "${BLUE}${BOLD}[STEP]${NORMAL}" "$1: $2"; }

error_exit() {
    error_message "$1"
    exit 1
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

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

remove_launch_agent() {
    info_message "Unloading $SERVER_NAME launch agent..."
    launchctl unload "$LAUNCH_AGENT_FILE" 2>/dev/null || true
    
    info_message "Deleting $LAUNCH_AGENT_FILE file..."
    rm -f "$LAUNCH_AGENT_FILE"
}

create_launch_agent_file() {
    local NAME=$1
    local FILE=$2
    local BIN_PATH=$3

    if [ -f "$FILE" ]; then
        info_message "Launch agent file $FILE already exists. Removing..."
        launchctl unload "$FILE" 2>/dev/null || true
        rm -f "$FILE"
        info_message "Old version of launch agent file removed successfully"
    fi

    echo "Creating launch agent file at $FILE..."

    cat > "$FILE" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.$NAME</string>
    <key>ProgramArguments</key>
    <array>
        <string>$BIN_PATH</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
EOF

    info_message "Launch agent file created for $NAME."
}

load_launch_agent() {
    local FILE=$1
    local NAME=$2
    info_message "Loading launch agent for $NAME..."
    launchctl load "$FILE"
    info_message "Launch agent loaded for $NAME."
}

# Main script execution
BIN_DIR="/usr/local/bin"
OS="darwin"
ARCH=$(uname -m)
case "$ARCH" in
    "x86_64") ARCH="amd64" ;;
    "arm64") ARCH="arm64" ;;
    *) error_exit "Unsupported architecture: $ARCH" ;;
esac

SERVER_BIN_NAME="$SERVER_NAME-${OS}-${ARCH}"
CLIENT_BIN_NAME="$CLIENT_NAME-${OS}-${ARCH}"
SERVER_BASE_URL="https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/download/v$WOPS_VERSION"
CLIENT_BASE_URL="https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/download/v$WOPS_VERSION"
SERVER_URL="$SERVER_BASE_URL/$SERVER_BIN_NAME"
CLIENT_URL="$CLIENT_BASE_URL/$CLIENT_BIN_NAME"

TEMP_DIR=$(mktemp -d) || error_exit "Failed to create temporary directory"
trap 'rm -rf "$TEMP_DIR"' EXIT

print_step 1 "Downloading binaries..."
curl -SL --progress-bar -o "$TEMP_DIR/$SERVER_BIN_NAME" "$SERVER_URL" || error_exit "Failed to download $SERVER_BIN_NAME"
curl -SL --progress-bar -o "$TEMP_DIR/$CLIENT_BIN_NAME" "$CLIENT_URL" || error_exit "Failed to download $CLIENT_BIN_NAME"

print_step 2 "Installing binaries to $BIN_DIR..."
maybe_sudo mv "$TEMP_DIR/$SERVER_BIN_NAME" "$BIN_DIR/$SERVER_NAME" || error_exit "Failed to move server binary to $BIN_DIR"
maybe_sudo chmod 755 "$BIN_DIR/$SERVER_NAME" || error_exit "Failed to set executable permissions on the server binary"
maybe_sudo mv "$TEMP_DIR/$CLIENT_BIN_NAME" "$BIN_DIR/$CLIENT_NAME" || error_exit "Failed to move client binary to $BIN_DIR"
maybe_sudo chmod 755 "$BIN_DIR/$CLIENT_NAME" || error_exit "Failed to set executable permissions on the client binary"

print_step 3 "Creating and loading launch agents..."
create_launch_agent_file "$SERVER_NAME" "$SERVER_LAUNCH_AGENT_FILE" "$BIN_DIR/$SERVER_NAME"
create_launch_agent_file "$CLIENT_NAME" "$CLIENT_LAUNCH_AGENT_FILE" "$BIN_DIR/$CLIENT_NAME"

load_launch_agent "$SERVER_LAUNCH_AGENT_FILE" "$SERVER_NAME"
load_launch_agent "$CLIENT_LAUNCH_AGENT_FILE" "$CLIENT_NAME"

success_message "Installation and configuration complete! Both $SERVER_NAME and $CLIENT_NAME are now set up as launch agents."
warn_message "You may need to restart your Mac or log out and back in for all changes to take effect."