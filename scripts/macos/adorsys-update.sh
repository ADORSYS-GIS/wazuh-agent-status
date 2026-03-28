#!/bin/bash
# Upgrade script from ADORSYS.
# Copyright (C) 2025, ADORSYS GmbH & CO KG.

# Check if we're running in bash; if not, adjust behavior
if [[ -n "$BASH_VERSION" ]]; then
    set -euo pipefail
else
    set -eu
fi

WAZUH_MANAGER=${WAZUH_MANAGER:-"wazuh.example.com"}
WAZUH_AGENT_REPO_REF=${WAZUH_AGENT_REPO_REF:-"main"}
SCRIPT_URL=${SCRIPT_URL:-"https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/${WAZUH_AGENT_REPO_REF}/scripts/macos/setup-agent.sh"}

# macOS-specific constants
ICON_PATH='/Library/Application Support/Ossec/wazuh-logo.png'
LOG_FILE='/Library/Ossec/logs/active-responses.log'
UPGRADE_SCRIPT_PATH='/Library/Ossec/active-response/bin/adorsys-update.sh'

# Determine architecture for macOS
ARCH=$(uname -m)
if [[ "$ARCH" = "x86_64" ]]; then
    BIN_FOLDER='/usr/local/bin'
else
    BIN_FOLDER='/opt/homebrew/bin'
fi

# Create a temporary directory
TMP_FOLDER=$(mktemp -d)

# Function for logging with timestamp
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp
    timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo -e "${timestamp} ${level} ${message}" >> "$LOG_FILE"
    return 0
}

# Logging helpers
info_message() {
    log "[INFO]" "$*"
    return 0
}

warn_message() {
    log "[WARNING]" "$*"
    return 0
}

error_message() {
    log "[ERROR]" "$*"
    return 0
}

cleanup() {
    if [[ -d "$TMP_FOLDER" ]]; then
        rm -rf "$TMP_FOLDER"
    fi
    return 0
}

trap cleanup EXIT

if [[ -f "$ICON_PATH" ]]; then
    ICON_ARG="with icon POSIX file \"$ICON_PATH\""
else
    warn_message "macOS icon file not found at '$ICON_PATH'. Sending notification without icon."
fi

send_notification() {
    local message="$1"
    local title="Wazuh Update"

    osascript -e "display dialog \"$message\" buttons {\"Dismiss\"} default button \"Dismiss\" with title \"$title\" $ICON_ARG"
    info_message "Notification sent: $message"
    return 0
}

# === Notify User with Action Dialog ===
PREPARE_MSG="A new version of Wazuh is available. Would you like to upgrade?"
ACTION=""

# Show dialog and capture user action, default to "Remind Me Later" if dismissed
ACTION=$(osascript <<EOF_OSASCRIPT
    try
        set userChoice to button returned of (display dialog "$PREPARE_MSG" buttons {"Remind Me Later", "Upgrade Now"} default button "Upgrade Now" with title "Wazuh Update" $ICON_ARG)
        return userChoice
    on error
        return "Remind Me Later"
    end try
EOF_OSASCRIPT
)

# --- Main Logic ---

info_message "Wazuh agent upgrade script started."

# Prompt the user for an action (Remind Me Later, Upgrade Now)
USER_ACTION="$ACTION"

run_upgrade() {
    info_message "Starting Wazuh agent upgrade..."
    info_message "Adding bin directory: $BIN_FOLDER to PATH environment"
    export PATH="$BIN_FOLDER:$PATH"
    info_message "Current PATH: $PATH"
    info_message "Starting setup. Using temporary directory: $TMP_FOLDER"

    # Check for required dependencies
    if ! command -v curl >/dev/null 2>&1; then
        error_message "curl is required but not installed."
        send_notification "Update failed: curl is missing."
        exit 1
    fi
    if ! command -v bash >/dev/null 2>&1; then
        error_message "bash is required but not installed."
        send_notification "Update failed: bash is missing."
        exit 1
    fi

    info_message "Downloading setup script..."
    if ! curl -SL -s "$SCRIPT_URL" -o "$TMP_FOLDER/setup-agent.sh" >> "$LOG_FILE"; then
        error_message "Failed to download setup-agent.sh"
        send_notification "Update failed: For more details go to file $LOG_FILE"
        exit 1
    fi

    chmod +x "$TMP_FOLDER/setup-agent.sh"

    if ! sudo WAZUH_MANAGER="$WAZUH_MANAGER" bash "$TMP_FOLDER/setup-agent.sh" >> "$LOG_FILE"; then
        error_message "Failed to setup wazuh agent"
        send_notification "Update failed: For more details go to file $LOG_FILE"
        exit 1
    fi

    send_notification "Update completed successfully! Please save your work and reboot your device to complete the update."
    return 0
}

case "$USER_ACTION" in
    "Remind Me Later")
        info_message "Update postponed. Exiting."
        exit 0
        ;;
    "Upgrade Now")
        info_message "User chose to update now."
        run_upgrade
        exit 0
        ;;
    *)
        info_message "Update postponed. Exiting."
        exit 0
        ;;
esac
