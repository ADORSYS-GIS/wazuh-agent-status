#!/bin/sh

# Set shell options
if [ -n "$BASH_VERSION" ]; then
    set -euo pipefail
else
    set -eu
fi

APP_VERSION=${APP_VERSION:-"0.5.0-rc.2"}

# Common configuration
SERVER_NAME=${SERVER_NAME:-"wazuh-agent-status"}
CLIENT_NAME=${CLIENT_NAME:-"wazuh-agent-status-client"}
WAZUH_AGENT_STATUS_REPO_REF=${WAZUH_AGENT_STATUS_REPO_REF:-"user-main"}
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
export CHECKSUMS_FILE="$CHECKSUMS_FILE"

# macOS-specific configuration
OS="darwin"
BIN_DIR="/usr/local/bin"
WAZUH_ACTIVE_RESPONSE_BIN_DIR="/Library/Ossec/active-response/bin"

ARCH=$(detect_architecture)
if [[ "$ARCH" != "amd64" ]] && [[ "$ARCH" != "arm64" ]]; then
    error_exit "Unsupported architecture: $ARCH. Only amd64 and arm64 are supported on macOS."
fi

# Environment Variables with Defaults
WAZUH_MANAGER=${WAZUH_MANAGER:-'wazuh.example.com'}

# Default to 'wazuh' user/group if they exist, otherwise fallback to root
USER_EXISTS=$(id -u wazuh 2>/dev/null || echo "")
GROUP_EXISTS=$(dscl . -list /Groups | grep -w "wazuh" || echo "")

if [ -n "$USER_EXISTS" ]; then
    WAZUH_USER=${WAZUH_USER:-"wazuh"}
else
    WAZUH_USER=${WAZUH_USER:-"root"}
fi

if [ -n "$GROUP_EXISTS" ]; then
    WAZUH_GROUP=${WAZUH_GROUP:-"wazuh"}
else
    WAZUH_GROUP=${WAZUH_GROUP:-"root"}
fi

SERVER_LAUNCH_AGENT_FILE=${SERVER_LAUNCH_AGENT_FILE:-"/Library/LaunchDaemons/com.adorsys.$SERVER_NAME.plist"}
CLIENT_LAUNCH_AGENT_FILE=${CLIENT_LAUNCH_AGENT_FILE:-"/Library/LaunchAgents/com.adorsys.$CLIENT_NAME.plist"}
MIGRATION_MARKER="/usr/local/etc/$SERVER_NAME/.migrated_from_go"

SERVER_BIN_NAME="$SERVER_NAME-$OS-$ARCH"
CLIENT_BIN_NAME="$CLIENT_NAME-$OS-$ARCH"
BASE_URL=${BASE_URL:-"https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/download/v$APP_VERSION"}
SERVER_URL="$BASE_URL/$SERVER_BIN_NAME"
CLIENT_URL="$BASE_URL/$CLIENT_BIN_NAME"
CHECKSUM_URL="$BASE_URL/checksums.sha256"

ADORSYS_UPDATE_SCRIPT_URL=${ADORSYS_UPDATE_SCRIPT_URL:-"$WAZUH_AGENT_STATUS_REPO_URL/scripts/macos/adorsys-update.sh"}
UPDATE_SCRIPT_PATH="$WAZUH_ACTIVE_RESPONSE_BIN_DIR/adorsys-update.sh"



# Legacy Go Cleanup
cleanup_legacy_system() {
    if [ -f "$MIGRATION_MARKER" ]; then
        info_message "Migration already completed. Skipping legacy cleanup."
        return 0
    fi
    print_step_header 0 "Legacy Go Cleanup"
    info_message "Detecting legacy Go components..."

    # 1. macOS: Unload legacy launchd plists
    info_message "Unloading legacy macOS launchd services..."
    maybe_sudo launchctl unload "$SERVER_LAUNCH_AGENT_FILE" 2>/dev/null || true
    maybe_sudo launchctl unload "$CLIENT_LAUNCH_AGENT_FILE" 2>/dev/null || true
    remove_file "$SERVER_LAUNCH_AGENT_FILE"
    remove_file "$CLIENT_LAUNCH_AGENT_FILE"

    # 2. Kill any lingering Go processes
    info_message "Killing lingering Go processes ($SERVER_NAME, $CLIENT_NAME)..."
    maybe_sudo killall "$SERVER_NAME" 2>/dev/null || true
    maybe_sudo killall "$CLIENT_NAME" 2>/dev/null || true

    info_message "Legacy cleanup complete."
    return 0
}

# macOS Launchd Plist File
create_launchd_plist_file() {
    local name="$1"
    local filepath="$2"

    info_message "Creating plist file for $name..."

    # Determine the EnvironmentVariables block: inject HOME only for the client (LaunchAgent)
    local env_dict_extra=""
    if [[ "$name" != "$SERVER_NAME" ]]; then
        local real_user=$(get_real_user)
        local user_home=$(eval echo "~$real_user")
        env_dict_extra="
        <key>HOME</key>
        <string>$user_home</string>"
    fi

    create_file "$filepath" "
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
    <key>Label</key>
    <string>com.adorsys.$name</string>
    <key>ProgramArguments</key>
    <array>
        <string>$BIN_DIR/$name</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>WAZUH_STATUS_LOG_FILE</key>
        <string>/var/log/wazuh-agent-status/wazuh-agent-status.log</string>$env_dict_extra
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
"

    local label="com.adorsys.$name"
    if [ "$name" = "$SERVER_NAME" ]; then
        local target="system/$label"
        if maybe_sudo launchctl print "$target" >/dev/null 2>&1; then
            info_message "Service $label is already loaded, kickstarting..."
            maybe_sudo launchctl kickstart -k "$target" 2>/dev/null || warn_message "Kickstarting server failed: $label"
        else
            info_message "Loading new daemon plist file..."
            maybe_sudo launchctl bootstrap system "$filepath" 2>/dev/null || warn_message "Loading server plist file failed: $filepath"
        fi
    else
        local real_user=$(get_real_user)
        local uid
        uid=$(id -u "$real_user")

        if sudo -u "$real_user" launchctl print "$target" >/dev/null 2>&1; then
            info_message "Service $label is already loaded, kickstarting..."
            sudo -u "$real_user" launchctl kickstart -k "$target" 2>/dev/null || warn_message "Kickstarting client failed: $label"
        else
            info_message "Loading $name into user GUI session ($real_user)..."
            # Use 'asuser' to ensure it's loaded in the correct GUI context
            sudo launchctl asuser "$uid" launchctl bootstrap "gui/$uid" "$filepath" 2>/dev/null || \
            sudo -u "$real_user" launchctl bootstrap "gui/$uid" "$filepath" 2>/dev/null || \
            warn_message "Loading $name failed. You may need to log out and log back in to see the tray app."
        fi
    fi

    info_message "macOS Launchd plist file created and managed: $filepath"
    return 0
}


# Startup Configurations
make_server_launch_at_startup() {
    create_launchd_plist_file "$SERVER_NAME" "$SERVER_LAUNCH_AGENT_FILE"
    return 0
}

make_client_launch_at_startup() {
    create_launchd_plist_file "$CLIENT_NAME" "$CLIENT_LAUNCH_AGENT_FILE"
    return 0
}

validate_installation() {
    # Validate binaries
    if [[ -x "$BIN_DIR/$SERVER_NAME" ]]; then
        success_message "Server binary exists and is executable: $BIN_DIR/$SERVER_NAME."
    else
        error_exit "Server binary is missing or not executable: $BIN_DIR/$SERVER_NAME."
    fi

    if [[ -x "$BIN_DIR/$CLIENT_NAME" ]]; then
        success_message "Client binary exists and is executable: $BIN_DIR/$CLIENT_NAME."
    else
        error_exit "Client binary is missing or not executable: $BIN_DIR/$CLIENT_NAME."
    fi

    # Validate service files
    if [[ -f "$SERVER_LAUNCH_AGENT_FILE" ]]; then
        success_message "macOS Launchd server plist exists: $SERVER_LAUNCH_AGENT_FILE."
    else
        error_exit "macOS Launchd server plist is missing: $SERVER_LAUNCH_AGENT_FILE."
    fi

    if [[ -f "$CLIENT_LAUNCH_AGENT_FILE" ]]; then
        success_message "macOS Launchd client plist exists: $CLIENT_LAUNCH_AGENT_FILE."
    else
        error_exit "macOS Launchd client plist is missing: $CLIENT_LAUNCH_AGENT_FILE."
    fi

    # Validate adorsys-update.sh script
    if maybe_sudo [ -f "$UPDATE_SCRIPT_PATH" ]; then
        success_message "adorsys-update.sh script exists: $UPDATE_SCRIPT_PATH."
    else
        error_exit "adorsys-update.sh script is missing: $UPDATE_SCRIPT_PATH."
    fi

    success_message "Installation complete!"
    return 0
}

print_step_header 0 "Legacy Go Cleanup"
cleanup_legacy_system

print_step_header 1 "Binaries Download"
info_message "Downloading server binary from $SERVER_URL..."
download_and_verify_file "$SERVER_URL" "$TMP_DIR/$SERVER_BIN_NAME" "$SERVER_BIN_NAME" "server binary" "$CHECKSUM_URL" || error_exit "Failed to download $SERVER_BIN_NAME"
info_message "Downloading client binary $CLIENT_URL..."
download_and_verify_file "$CLIENT_URL" "$TMP_DIR/$CLIENT_BIN_NAME" "$CLIENT_BIN_NAME" "client binary" "$CHECKSUM_URL" || error_exit "Failed to download $CLIENT_BIN_NAME"
success_message "Binaries downloaded successfully."

print_step_header 2 "Binaries Installation"
info_message "Create Binary directory $BIN_DIR if it doesn't exist"
maybe_sudo mkdir -p "$BIN_DIR" || error_exit "Failed to create directory $BIN_DIR"
info_message "Installing server binary to $BIN_DIR..."
maybe_sudo mv "$TMP_DIR/$SERVER_BIN_NAME" "$BIN_DIR/$SERVER_NAME"
maybe_sudo chmod +x "$BIN_DIR/$SERVER_NAME"
info_message "Installing client binary to $BIN_DIR..."
maybe_sudo mv "$TMP_DIR/$CLIENT_BIN_NAME" "$BIN_DIR/$CLIENT_NAME"
maybe_sudo chmod +x "$BIN_DIR/$CLIENT_NAME"
success_message "Binaries installed successfully."

print_step_header 3 "Server Service Configuration"
make_server_launch_at_startup

print_step_header 4 "Client Service Configuration"
make_client_launch_at_startup

print_step_header 5 "Download and configure adorsys-update.sh"
info_message "Downloading adorsys-update.sh..."
if maybe_sudo [ -d "$WAZUH_ACTIVE_RESPONSE_BIN_DIR" ]; then
    download_and_verify_file "$ADORSYS_UPDATE_SCRIPT_URL" "$UPDATE_SCRIPT_PATH" "scripts/macos/adorsys-update.sh" "adorsys-update.sh script" "$WAZUH_AGENT_STATUS_REPO_URL/checksums.sha256" || warn_message "Failed to download adorsys-update.sh"
    maybe_sudo chmod 750 "$UPDATE_SCRIPT_PATH"

    # Update WAZUH_MANAGER value in adorsys-update.sh
    if [[ -n "${WAZUH_MANAGER:-}" ]]; then
        info_message "Updating WAZUH_MANAGER in adorsys-update.sh to $WAZUH_MANAGER"
        maybe_sudo sed_inplace "s/^WAZUH_MANAGER=.*/WAZUH_MANAGER=\${WAZUH_MANAGER:-\"$WAZUH_MANAGER\"}/" "$UPDATE_SCRIPT_PATH"
    else
        warn_message "WAZUH_MANAGER variable not set. Skipping update in adorsys-update.sh."
    fi
else
    warn_message "$WAZUH_ACTIVE_RESPONSE_BIN_DIR does not exist, skipping."
fi

# Permissions and Ownership Configuration
print_step_header 6 "Permissions and Ownership Configuration"
setup_permissions_and_ownership "$WAZUH_USER" "$WAZUH_GROUP" "/Library/Ossec/bin/wazuh-control"

print_step_header 7 "Validating installation and configuration..."
validate_installation

# Create migration marker
maybe_sudo mkdir -p "$(dirname "$MIGRATION_MARKER")"
maybe_sudo touch "$MIGRATION_MARKER"
