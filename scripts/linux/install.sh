#!/bin/sh

# Set shell options
if [ -n "$BASH_VERSION" ]; then
    set -euo pipefail
else
    set -eu
fi

# OS guard early in the script
if [[ "$(uname -s)" != "Linux" ]]; then
    printf "%s\n" "[ERROR] This installation script is intended for Linux systems. Please use the appropriate script for your operating system." >&2
    exit 1
fi

PROFILE=${PROFILE:-"user"}
APP_VERSION=${APP_VERSION:-"0.4.2.rc1"}

# Assign app version based on profile
case "$PROFILE" in
"admin") WAS_VERSION="$APP_VERSION" ;;
*) WAS_VERSION="$APP_VERSION-user" ;;
esac

# Common configuration
WAZUH_AGENT_STATUS_REPO_REF=${WAZUH_AGENT_STATUS_REPO_REF:-"refs/tags/v$WAS_VERSION"}
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

# Linux-specific configuration
OS="linux"
BIN_DIR="/usr/local/bin"
WAZUH_ACTIVE_RESPONSE_BIN_DIR="/var/ossec/active-response/bin"

ARCH=$(detect_architecture)
if [[ "$ARCH" != "amd64" ]]; then
    error_exit "Unsupported architecture: $ARCH"
fi

# Environment Variables with Defaults
SERVER_NAME=${SERVER_NAME:-"wazuh-agent-status"}
CLIENT_NAME=${CLIENT_NAME:-"wazuh-agent-status-client"}
WAZUH_MANAGER=${WAZUH_MANAGER:-'wazuh.example.com'}
WAZUH_USER=${WAZUH_USER:-"root"}

SERVICE_FILE=${SERVICE_FILE:-"/etc/systemd/system/$SERVER_NAME.service"}
DESKTOP_UNIT_FOLDER=${DESKTOP_UNIT_FOLDER:-"$HOME/.config/autostart"}
DESKTOP_UNIT_FILE=${DESKTOP_UNIT_FILE:-"$DESKTOP_UNIT_FOLDER/$CLIENT_NAME.desktop"}

SERVER_BIN_NAME="$SERVER_NAME-$OS-$ARCH"
CLIENT_BIN_NAME="$CLIENT_NAME-$OS-$ARCH"
BASE_URL=${BASE_URL:-"https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/download/v$WAS_VERSION"}
SERVER_URL="$BASE_URL/$SERVER_BIN_NAME"
CLIENT_URL="$BASE_URL/$CLIENT_BIN_NAME"
CHECKSUM_URL="$BASE_URL/checksums.sha256"

ADORSYS_UPDATE_SCRIPT_URL=${ADORSYS_UPDATE_SCRIPT_URL:-"$WAZUH_AGENT_STATUS_REPO_URL/scripts/linux/adorsys-update.sh"}
UPDATE_SCRIPT_PATH="$WAZUH_ACTIVE_RESPONSE_BIN_DIR/adorsys-update.sh"

# Service Management
create_service_file() {
    info_message "Removing old service file if it exists..."
    remove_file "$SERVICE_FILE"

    info_message "Creating a new systemd service file..."
    create_file "$SERVICE_FILE" "
[Unit]
Description=Wazuh Agent Status daemon
After=network.target

[Service]
ExecStart=$BIN_DIR/$SERVER_NAME
Restart=always
User=$WAZUH_USER

[Install]
WantedBy=multi-user.target
"
    info_message "Systemd service file created: $SERVICE_FILE"
    return 0
}

reload_and_enable_service() {
    info_message "Reloading systemd daemon..."
    maybe_sudo systemctl daemon-reload

    info_message "Enabling service to start at boot..."
    maybe_sudo systemctl enable "$SERVER_NAME"

    info_message "Starting the service..."
    maybe_sudo systemctl start "$SERVER_NAME"

    info_message "Systemd service enabled and started."
    return 0
}

# Desktop Unit File Creation
create_desktop_unit_file() {
    info_message "Creating desktop unit directory if it doesn't exist..."
    mkdir -p "$DESKTOP_UNIT_FOLDER"

    info_message "Creating desktop unit file for autostart..."
    create_file "$DESKTOP_UNIT_FILE" "
[Desktop Entry]
Name=Wazuh Agent Monitoring Tray Icon App
GenericName=Script for GNOME startup
Comment=Runs the tray script
Exec=$BIN_DIR/$CLIENT_NAME
Terminal=false
Type=Application
X-GNOME-Autostart-enabled=true
"
    info_message "Desktop autostart file created: $DESKTOP_UNIT_FILE"
    return 0
}

# Startup Configurations
make_server_launch_at_startup() {
    create_service_file && reload_and_enable_service
    return 0
}

make_client_launch_at_startup() {
    create_desktop_unit_file
    return 0
}

sed_inplace() {
    maybe_sudo sed -i "$@" 2>/dev/null || true
    return $?
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
    if [[ -f "$SERVICE_FILE" ]]; then
        success_message "Systemd service file exists: $SERVICE_FILE."
    else
        error_exit "Systemd service file is missing: $SERVICE_FILE."
    fi

    systemctl is-enabled "$SERVER_NAME" >/dev/null 2>&1 &&
        success_message "Systemd service is enabled: $SERVER_NAME." ||
        error_exit "Systemd service is not enabled: $SERVER_NAME."

    if [[ -f "$DESKTOP_UNIT_FILE" ]]; then
        success_message "Desktop autostart file exists: $DESKTOP_UNIT_FILE."
    else
        error_exit "Desktop autostart file is missing: $DESKTOP_UNIT_FILE."
    fi
    
    # Validate adorsys-update.sh script
    if maybe_sudo [ -f "$UPDATE_SCRIPT_PATH" ]; then
        success_message "adorsys-update.sh script exists: $UPDATE_SCRIPT_PATH."
    else
        error_exit "adorsys-update.sh script is missing: $UPDATE_SCRIPT_PATH."
    fi

    success_message "Installation complete! Restart your system to apply changes for the wazuh agent status."
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
make_client_launch_at_startup

print_step_header 5 "Download and configure adorsys-update.sh"
info_message "Downloading adorsys-update.sh..."
if maybe_sudo [ -d "$WAZUH_ACTIVE_RESPONSE_BIN_DIR" ]; then
    download_and_verify_file "$ADORSYS_UPDATE_SCRIPT_URL" "$UPDATE_SCRIPT_PATH" "scripts/linux/adorsys-update.sh" "adorsys-update.sh script" "$WAZUH_AGENT_STATUS_REPO_URL/checksums.sha256" || warn_message "Failed to download adorsys-update.sh"
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
