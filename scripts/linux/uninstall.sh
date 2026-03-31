#!/bin/sh

# Set shell options
if [ -n "$BASH_VERSION" ]; then
    set -euo pipefail
else
    set -eu
fi

# Environment Variables with Defaults
SERVER_NAME=${SERVER_NAME:-"wazuh-agent-status"}
CLIENT_NAME=${CLIENT_NAME:-"wazuh-agent-status-client"}
SERVICE_FILE="/etc/systemd/system/$SERVER_NAME.service"
DESKTOP_UNIT_FILE="$HOME/.config/autostart/$CLIENT_NAME.desktop"
BIN_DIR="/usr/local/bin"

# Linux-specific configuration
OS="linux"
UPGRADE_SCRIPT_PATH="/var/ossec/active-response/bin/adorsys-update.sh"

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

# Check if sudo is available or if the script is run as root
maybe_sudo() {
    if [ "$(id -u)" -ne 0 ]; then
        if command -v sudo >/dev/null 2>&1; then
            sudo "$@"
        else
            error_message "This script requires root privileges. Please run with sudo or as root."
            exit 1
        fi
    else
        "$@"
    fi
}

# Remove file if it exists
remove_file() {
    local filepath="$1"
    if maybe_sudo [ -f "$filepath" ]; then
        info_message "Removing file: $filepath"
        maybe_sudo rm -f "$filepath"
    else
        warn_message "File not found: $filepath. Skipping."
    fi
}

# Remove binary files
remove_binaries() {
    info_message "Removing binaries from $BIN_DIR..."
    remove_file "$BIN_DIR/$SERVER_NAME"
    remove_file "$BIN_DIR/$CLIENT_NAME"
}

# Remove Linux systemd service
remove_systemd_service() {
    if [ -f "$SERVICE_FILE" ]; then
        info_message "Disabling and removing systemd service..."
        maybe_sudo systemctl stop "$SERVER_NAME" || true
        maybe_sudo systemctl disable "$SERVER_NAME" || true
        remove_file "$SERVICE_FILE"
        maybe_sudo systemctl daemon-reload
    else
        warn_message "Systemd service not found or not running. Skipping."
    fi
}

# Remove Linux desktop unit file for autostart
remove_desktop_unit() {
    if [ -f "$DESKTOP_UNIT_FILE" ]; then
        info_message "Removing desktop unit file for autostart..."
        remove_file "$DESKTOP_UNIT_FILE"
    else
        warn_message "Desktop unit file not found. Skipping."
    fi
}

# Uninstallation Process
remove_binaries
remove_file "$UPGRADE_SCRIPT_PATH"
remove_systemd_service
remove_desktop_unit

success_message "Wazuh agent status uninstalled completed successfully."
