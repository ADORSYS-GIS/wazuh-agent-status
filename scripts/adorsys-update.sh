#!/bin/bash
# Upgrade script from ADORSYS.
# Copyright (C) 2025, ADORSYS GmbH & CO KG.

# Check if we're running in bash; if not, adjust behavior
if [ -n "$BASH_VERSION" ]; then
    set -euo pipefail
else
    set -eu
fi

LOG_LEVEL=${LOG_LEVEL:-'INFO'}
WAZUH_MANAGER=${WAZUH_MANAGER:-"wazuh.example.com"}
SCRIPT_URL=${SCRIPT_URL:-"https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/main/scripts/setup-agent.sh"}
OS_TYPE=$(uname -s)

USER=""
USER_UID=""
DBUS_PATH=""
ICON_ARG=""

if [ "$OS_TYPE" = "Darwin" ]; then
    ICON_PATH='/Library/Application Support/Ossec/wazuh-logo.png'
    LOG_FILE='/Library/Ossec/logs/active-responses.log'
    UPGRADE_SCRIPT_PATH='/Library/Ossec/active-response/bin/adorsys-update.sh'
    ARCH=$(uname -m)
    if [ "$ARCH" = "x86_64" ]; then
        BIN_FOLDER='/usr/local/bin'
    else
        BIN_FOLDER='/opt/homebrew/bin'
    fi
else
    ICON_PATH='/usr/share/pixmaps/wazuh-logo.png'
    LOG_FILE='/var/ossec/logs/active-responses.log'
    UPGRADE_SCRIPT_PATH='/var/ossec/active-response/bin/adorsys-update.sh'
    BIN_FOLDER='/usr/bin'
fi

# Create a temporary directory
TMP_FOLDER=$(mktemp -d)

# --- Start of Centralized User Detection ---

# Get the currently logged-in user
get_logged_in_user() {
    who | awk '{print $1}' | head -n 1
}

if [ "$OS_TYPE" = "Linux" ]; then
    # Find the user logged into the primary graphical display (:0)
    USER=$(get_logged_in_user)
    if [ -n "$USER" ]; then
        USER_UID=$(id -u "$USER")
        if [ -n "$USER_UID" ]; then
            DBUS_PATH="/run/user/$USER_UID/bus"
        fi
    fi
fi
# --- End of Centralized User Detection ---

# Function for logging with timestamp
log() {
    local LEVEL="$1"
    shift
    local MESSAGE="$*"
    local TIMESTAMP
    TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
    echo -e "${TIMESTAMP} ${LEVEL} ${MESSAGE}" >> "$LOG_FILE"
}

# Logging helpers
info_message() {
    log "[INFO]" "$*"
}

warning_message() {
    log "[WARNING]" "$*"
}

error_message() {
    log "[ERROR]" "$*"
}

cleanup() {
    if [ -d "$TMP_FOLDER" ]; then
        rm -rf "$TMP_FOLDER"
    fi
}

trap cleanup EXIT

if [ "$OS_TYPE" = "Darwin" ]; then
    if [ -f "$ICON_PATH" ]; then
        ICON_ARG="with icon POSIX file \"$ICON_PATH\""
    else
        warn_message "macOS icon file not found at '$ICON_PATH'. Sending notification without icon."
    fi
else
    if [ -f "$ICON_PATH" ]; then
        ICON_ARG="-i $ICON_PATH"
    else
        warn_message "Linux icon file not found at '$ICON_PATH'. Sending notification without icon."
    fi
fi

send_notification() {
    local message="$1"
    local title="Wazuh Update"

    if [ "$OS_TYPE" = "Darwin" ]; then
        osascript -e "display dialog \"$message\" buttons {\"Dismiss\"} default button \"Dismiss\" with title \"$title\" $ICON_ARG"
    elif [ "$OS_TYPE" = "Linux" ]; then
        sudo -u "$USER" DISPLAY=:0 DBUS_SESSION_BUS_ADDRESS="unix:path=$DBUS_PATH" \
                notify-send --app-name=Wazuh -u critical "$title" "$message" $ICON_ARG
    else
        error_message "Unsupported OS for notifications: $(uname)"
    fi
    info_message "Notification sent: $message"
}

# === Notify User with Action Dialog ===
PREPARE_MSG="A new version of Wazuh is available. Would you like to upgrade?"
ACTION=""

if [ "$OS_TYPE" = "Darwin" ]; then
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
elif [ "$OS_TYPE" = "Linux" ]; then
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
else
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