#!/bin/sh

# Set shell options
if [ -n "$BASH_VERSION" ]; then
    set -euo pipefail
else
    set -eu
fi

# Common configuration
WAZUH_AGENT_STATUS_REPO_REF=${WAZUH_AGENT_STATUS_REPO_REF:-"v0.4.2"}
WAZUH_AGENT_STATUS_REPO_URL="https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$WAZUH_AGENT_STATUS_REPO_REF"

# Source shared utilities
TMP_DIR=$(mktemp -d)
export CHECKSUMS_FILE="$TMP_DIR/checksums.sha256"
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
if ! curl -fsSL "${WAZUH_AGENT_STATUS_REPO_URL}/checksums.sha256" -o "$CHECKSUMS_FILE"; then
    echo "Failed to download checksums.sha256"
    exit 1
fi

EXPECTED_HASH=$(grep "scripts/shared/utils.sh" "$CHECKSUMS_FILE" | awk '{print $1}')
ACTUAL_HASH=$(calculate_sha256_bootstrap "$TMP_DIR/utils.sh")

if [[ -z "$EXPECTED_HASH" ]] || [[ "$EXPECTED_HASH" != "$ACTUAL_HASH" ]]; then
    echo "Error: Checksum verification failed for utils.sh" >&2
    echo "Expected hash: $EXPECTED_HASH" >&2
    echo "Actual hash: $ACTUAL_HASH" >&2
    exit 1
fi

. "$TMP_DIR/utils.sh"

trap cleanup EXIT

# Environment Variables with Defaults
SERVER_NAME=${SERVER_NAME:-"wazuh-agent-status"}
CLIENT_NAME=${CLIENT_NAME:-"wazuh-agent-status-client"}
SERVER_LAUNCH_AGENT_FILE="/Library/LaunchDaemons/com.adorsys.$SERVER_NAME.plist"
CLIENT_LAUNCH_AGENT_FILE="/Library/LaunchAgents/com.adorsys.$CLIENT_NAME.plist"
BIN_DIR="/usr/local/bin"
OS="darwin"
UPGRADE_SCRIPT_PATH="/Library/Ossec/active-response/bin/adorsys-update.sh"

# Remove file if it exists
remove_file() {
    local filepath="$1"
    if maybe_sudo [ -f "$filepath" ]; then
        info_message "Removing file: $filepath"
        maybe_sudo rm -f "$filepath"
    else
        warn_message "File not found: $filepath. Skipping."
    fi
    return 0
}

# Remove binary files
remove_binaries() {
    info_message "Removing binaries from $BIN_DIR..."
    remove_file "$BIN_DIR/$SERVER_NAME"
    remove_file "$BIN_DIR/$CLIENT_NAME"
    return 0
}

# Remove macOS Launchd plist files
remove_launchd_service() {
    local name="$1"
    local filepath="$2"
    if [[ -f "$filepath" ]]; then
        info_message "Unloading and removing Launchd plist for $name..."
        maybe_sudo launchctl unload "$filepath" 2>/dev/null || true
        remove_file "$filepath"
    else
        warn_message "Launchd service for $name not found. Skipping."
    fi
    return 0
}

# Uninstallation Process
remove_binaries
remove_file "$UPGRADE_SCRIPT_PATH"
remove_launchd_service "$SERVER_NAME" "$SERVER_LAUNCH_AGENT_FILE"
remove_launchd_service "$CLIENT_NAME" "$CLIENT_LAUNCH_AGENT_FILE"

success_message "Wazuh agent status uninstalled completed successfully."
