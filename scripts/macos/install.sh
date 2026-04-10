#!/bin/sh

# Set shell options
if [ -n "$BASH_VERSION" ]; then
    set -euo pipefail
else
    set -eu
fi

PROFILE=${PROFILE:-"user"}
APP_VERSION=${APP_VERSION:-"0.4.2"}

# Assign app version based on profile
case "$PROFILE" in
"admin") WAS_VERSION="$APP_VERSION" ;;
*) WAS_VERSION="$APP_VERSION-user" ;;
esac

# Common configuration
SERVER_NAME=${SERVER_NAME:-"wazuh-agent-status"}
CLIENT_NAME=${CLIENT_NAME:-"wazuh-agent-status-client"}
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
WAZUH_USER=${WAZUH_USER:-"root"}

SERVER_LAUNCH_AGENT_FILE=${SERVER_LAUNCH_AGENT_FILE:-"/Library/LaunchDaemons/com.adorsys.$SERVER_NAME.plist"}
CLIENT_LAUNCH_AGENT_FILE=${CLIENT_LAUNCH_AGENT_FILE:-"/Library/LaunchAgents/com.adorsys.$CLIENT_NAME.plist"}

SERVER_BIN_NAME="$SERVER_NAME-$OS-$ARCH"
CLIENT_BIN_NAME="$CLIENT_NAME-$OS-$ARCH"
BASE_URL=${BASE_URL:-"https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/download/v$WAS_VERSION"}
SERVER_URL="$BASE_URL/$SERVER_BIN_NAME"
CLIENT_URL="$BASE_URL/$CLIENT_BIN_NAME"
CHECKSUM_URL="$BASE_URL/checksums.sha256"

ADORSYS_UPDATE_SCRIPT_URL=${ADORSYS_UPDATE_SCRIPT_URL:-"$WAZUH_AGENT_STATUS_REPO_URL/scripts/macos/adorsys-update.sh"}
UPDATE_SCRIPT_PATH="$WAZUH_ACTIVE_RESPONSE_BIN_DIR/adorsys-update.sh"


remove_file() {
    local filepath="$1"
    if [[ -f "$filepath" ]]; then
        info_message "Removing file: $filepath"
        maybe_sudo rm -f "$filepath"
    fi
    return 0
}

# macOS Launchd Plist File
create_launchd_plist_file() {
    local name="$1"
    local filepath="$2"

    info_message "Creating plist file for $name..."
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
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
"
    
    if [[ "$name" = "$SERVER_NAME" ]]; then
        info_message "Loading new daemon plist file..."
        maybe_sudo launchctl bootstrap "system $filepath" 2>/dev/null || warn_message "loading previous plist file failed: $filepath"
    else
        info_message "Loading new agent plist file..."
        launchctl bootstrap "gui/$(id) $filepath" 2>/dev/null || warn_message "loading previous plist file failed: $filepath"
    fi
    info_message "macOS Launchd plist file created and loaded: $filepath"
    return 0
}

unload_plist_file() {
    local filepath="$1"

    if [[ -f "$filepath" ]]; then
        info_message "Unloading previous plist file (if any)..."
        maybe_sudo launchctl bootout "gui/$(id) $filepath" 2>/dev/null || warn_message "Unloading previous plist file failed: $filepath"
        info_message "Previous plist file unloaded: $filepath"
    else
        warn_message "Plist file: $filepath does not exist. Skipping."
    fi
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
unload_plist_file "$CLIENT_LAUNCH_AGENT_FILE"
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

print_step_header 6 "Validating installation and configuration..."
validate_installation
