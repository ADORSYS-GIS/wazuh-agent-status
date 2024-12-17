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
SERVER_LAUNCH_AGENT_FILE="/Library/LaunchDaemons/com.adorsys.$SERVER_NAME.plist"
CLIENT_LAUNCH_AGENT_FILE="/Library/LaunchAgents/com.adorsys.$CLIENT_NAME.plist"
DESKTOP_UNIT_FILE="$HOME/.config/autostart/$CLIENT_NAME.desktop"
BIN_DIR="/usr/local/bin"

# OS Detection
case "$(uname)" in
    Linux) OS="linux" ;;
    Darwin) OS="darwin" ;;
    *) echo "Unsupported operating system: $(uname)" >&2; exit 1 ;;
esac

# Logging Utilities
info_message() { echo "[INFO] $*"; }
warn_message() { echo "[WARNING] $*"; }
error_message() { echo "[ERROR] $*"; }
success_message() { echo "[SUCCESS] $*"; }

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

# Remove file if it exists
remove_file() {
    local filepath="$1"
    if [ -f "$filepath" ]; then
        info_message "Removing file: $filepath"
        maybe_sudo rm -f "$filepath"
    else
        warn_message "File not found: $filepath"
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
    if [ "$OS" = "linux" ] && [ -f "$SERVICE_FILE" ]; then
        info_message "Disabling and removing systemd service..."
        maybe_sudo systemctl stop "$SERVER_NAME" || true
        maybe_sudo systemctl disable "$SERVER_NAME" || true
        remove_file "$SERVICE_FILE"
        maybe_sudo systemctl daemon-reload
    fi
}

# Remove macOS Launchd plist files
remove_launchd_service() {
    local name="$1"
    local filepath="$2"
    if [ "$OS" = "darwin" ] && [ -f "$filepath" ]; then
        info_message "Unloading and removing Launchd plist for $name..."
        maybe_sudo launchctl unload "$filepath" 2>/dev/null || true
        remove_file "$filepath"
    fi
}

# Remove Linux desktop unit file for autostart
remove_desktop_unit() {
    if [ -f "$DESKTOP_UNIT_FILE" ]; then
        info_message "Removing desktop unit file for autostart..."
        remove_file "$DESKTOP_UNIT_FILE"
    fi
}

# Uninstallation Process
info_message "Step 1: Removing binaries..."
remove_binaries

info_message "Step 2: Removing services..."
remove_systemd_service
remove_launchd_service "$SERVER_NAME" "$SERVER_LAUNCH_AGENT_FILE"
remove_launchd_service "$CLIENT_NAME" "$CLIENT_LAUNCH_AGENT_FILE"

info_message "Step 3: Removing autostart configuration..."
remove_desktop_unit

success_message "Uninstallation of wazuh-agent-status completed successfully."
