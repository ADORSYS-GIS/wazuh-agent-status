#!/bin/bash
# Upgrade script from ADORSYS.
# Copyright (C) 2025, ADORSYS GmbH & CO KG.

# Check if we're running in bash; if not, adjust behavior
if [[ -n "$BASH_VERSION" ]]; then
    set -euo pipefail
else
    set -eu
fi

# OS guard early in the script
if [[ "$(uname -s)" != "Darwin" ]]; then
    printf "%s\n" "[ERROR] This installation script is intended for macOS systems. Please use the appropriate script for your operating system." >&2
    exit 1
fi

WAZUH_AGENT_STATUS_REPO_REF=${WAZUH_AGENT_STATUS_REPO_REF:-"v0.4.2"}
WAZUH_AGENT_STATUS_REPO_URL="https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$WAZUH_AGENT_STATUS_REPO_REF"

# Source shared utilities
if ! curl -fsSL "${WAZUH_AGENT_STATUS_REPO_URL}/scripts/shared/utils.sh" -o "$TMP_DIR/utils.sh"; then
    echo "Failed to download utils.sh"
    exit 1
fi

# Function to calculate SHA256 (cross-platform bootstrap)
calculate_sha256_bootstrap() {
    local file="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{print $1}'
    else
        shasum -a 256 "$file" | awk '{print $1}'
    fi
    return 0
}

# Download checksums and verify utils.sh integrity BEFORE sourcing it
if ! curl -fsSL "${WAZUH_AGENT_STATUS_REPO_URL}/checksums.sha256" -o "$TMP_DIR/checksums.sha256"; then
    echo "Failed to download checksums.sha256"
    exit 1
fi

EXPECTED_HASH=$(grep "scripts/shared/utils.sh" "$TMP_DIR/checksums.sha256" | awk '{print $1}')
ACTUAL_HASH=$(calculate_sha256_bootstrap "$TMP_DIR/utils.sh")

if [[ -z "$EXPECTED_HASH" ]] || [[ "$EXPECTED_HASH" != "$ACTUAL_HASH" ]]; then
    echo "Error: Checksum verification failed for utils.sh" >&2
    echo "Expected hash: $EXPECTED_HASH" >&2
    echo "Actual hash: $ACTUAL_HASH" >&2
    exit 1
fi

. "$TMP_DIR/utils.sh"

trap cleanup EXIT
export CHECKSUMS_FILE="$CHECKSUMS_FILE"

# Environment Variables with Defaults
WAZUH_MANAGER=${WAZUH_MANAGER:-"wazuh.example.com"}
WAZUH_AGENT_REPO_REF=${WAZUH_AGENT_REPO_REF:-"main"}
SCRIPT_URL=${SCRIPT_URL:-"https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/${WAZUH_AGENT_REPO_REF}/scripts/macos/setup-agent.sh"}

# macOS-specific constants
ICON_PATH='/Library/Application Support/Ossec/wazuh-logo.png'
LOG_FILE='/Library/Ossec/logs/active-responses.log'
UPGRADE_SCRIPT_PATH='/Library/Ossec/active-response/bin/adorsys-update.sh'

# Determine architecture for macOS
ARCH=$(detect_architecture)
if [[ "$ARCH" = "amd64" ]]; then
    BIN_FOLDER='/usr/local/bin'
else
    BIN_FOLDER='/opt/homebrew/bin'
fi

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
    info_message "Starting setup. Using temporary directory: $TMP_DIR"

    # Check for required dependencies
    if ! command -v curl >/dev/null 2>&1; then
        send_notification "Update failed: curl is missing."
        error_exit "curl is required but not installed."
    fi
    if ! command -v bash >/dev/null 2>&1; then
        send_notification "Update failed: bash is missing."
        error_exit "bash is required but not installed."
    fi

    info_message "Downloading setup script..."
    if ! download_and_verify_file "${WAZUH_AGENT_REPO_URL}/scripts/macos/setup-agent.sh" "$TMP_DIR/setup-agent.sh" "scripts/macos/setup-agent.sh" "setup-agent.sh" "$WAZUH_AGENT_REPO_URL/checksums.sha256"; then
        send_notification "Update failed: For more details go to file $LOG_FILE"
        error_exit "Failed to download setup-agent.sh"
    fi

    maybe_sudo chmod +x "$TMP_DIR/setup-agent.sh"

    if ! sudo env WAZUH_MANAGER="$WAZUH_MANAGER" bash "$TMP_DIR/setup-agent.sh" >> "$LOG_FILE"; then
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
