#!/bin/bash
# Linux upgrade script from ADORSYS.
# Copyright (C) 2025, ADORSYS GmbH & CO KG.

# Check if we're running in bash; if not, adjust behavior
if [[ -n "$BASH_VERSION" ]]; then
    set -euo pipefail
else
    set -eu
fi

# OS guard early in the script
if [[ "$(uname -s)" != "Linux" ]]; then
    printf "%s\n" "[ERROR] This installation script is intended for Linux systems. Please use the appropriate script for your operating system." >&2
    exit 1
fi

WAZUH_AGENT_STATUS_REPO_REF=${WAZUH_AGENT_STATUS_REPO_REF:-"main"}
WAZUH_AGENT_STATUS_REPO_URL="https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$WAZUH_AGENT_STATUS_REPO_REF"

# Source shared utilities
TMP_DIR=$(mktemp -d)
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
WAZUH_AGENT_REPO_URL="https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/$WAZUH_AGENT_REPO_REF"

# Linux-specific constants
ICON_PATH='/usr/share/pixmaps/wazuh-logo.png'
LOG_FILE='/var/ossec/logs/active-responses.log'
UPGRADE_SCRIPT_PATH='/var/ossec/active-response/bin/adorsys-update.sh'
BIN_FOLDER='/usr/bin'

# --- Start of Centralized User Detection ---

# Get the currently logged-in user
get_logged_in_user() {
    who | awk '{print $1}' | head -n 1
    return 0
}

# Find the user logged into the primary graphical display (:0)
USER=$(get_logged_in_user)
if [[ -n "$USER" ]]; then
    USER_UID=$(id -u "$USER")
    if [[ -n "$USER_UID" ]]; then
        DBUS_PATH="/run/user/$USER_UID/bus"
    fi
fi
# --- End of Centralized User Detection ---

if [[ -f "$ICON_PATH" ]]; then
    ICON_ARG="-i $ICON_PATH"
else
    warn_message "Icon file not found at '$ICON_PATH'. Sending notification without icon."
fi

send_notification() {
    local message="$1"
    local title="Wazuh Update"

    sudo -u "$USER" DISPLAY=:0 DBUS_SESSION_BUS_ADDRESS="unix:path=$DBUS_PATH" \
            notify-send --app-name=Wazuh -u critical "$title" "$message" $ICON_ARG
    info_message "Notification sent: $message"
    return 0
}

# === Notify User with Action Dialog ===
PREPARE_MSG="A new version of Wazuh is available. Would you like to upgrade?"
ACTION=""

if notify-send --help 2>&1 | grep -q -- '--action'; then
    NOTIFY_COMMAND=(sudo -u "$USER" DISPLAY=:0 DBUS_SESSION_BUS_ADDRESS="unix:path=$DBUS_PATH" notify-send --app-name=Wazuh -u critical)
    NOTIFY_COMMAND+=( $ICON_ARG )

    NOTIFY_COMMAND+=( -A "Remind Me Later=Remind Me Later" -A "Upgrade Now=Upgrade Now" )
    NOTIFY_COMMAND+=( "Wazuh Update" "$PREPARE_MSG" )
    # Execute notify-send and capture its output (the action ID)
    ACTION=$("${NOTIFY_COMMAND[@]}" 2>/dev/null)
else
    # Fallback to notify-send, cannot capture user response
    sudo -u "$USER" DISPLAY=:0 DBUS_SESSION_BUS_ADDRESS="unix:path=$DBUS_PATH" \
        notify-send --app-name=Wazuh -u critical $ICON_ARG \
        "Wazuh Update" "$PREPARE_MSG"
    warn_message "notify-send does not support actions"
    ACTION="Remind Me Later"
fi

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
    if ! download_and_verify_file "${WAZUH_AGENT_REPO_URL}/scripts/linux/setup-agent.sh" "$TMP_DIR/setup-agent.sh" "scripts/linux/setup-agent.sh" "setup-agent.sh" "$WAZUH_AGENT_REPO_URL/checksums.sha256"; then
        send_notification "Update failed: For more details go to file $LOG_FILE"
        error_exit "Failed to download setup-agent.sh"
    fi

    maybe_sudo chmod +x "$TMP_DIR/setup-agent.sh"

    if ! maybe_sudo env WAZUH_MANAGER="$WAZUH_MANAGER" bash "$TMP_DIR/setup-agent.sh" >> "$LOG_FILE"; then
        send_notification "Update failed: For more details go to file $LOG_FILE"
        error_exit "Failed to setup wazuh agent"
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
