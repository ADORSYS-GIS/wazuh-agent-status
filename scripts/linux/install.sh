#!/usr/bin/env bash

# Set shell options
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

APP_VERSION=${APP_VERSION:-"0.5.2-rc.1"}

# Common configuration
WAZUH_AGENT_STATUS_REPO_REF=${WAZUH_AGENT_STATUS_REPO_REF:-"main"}
WAZUH_AGENT_STATUS_REPO_URL="https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$WAZUH_AGENT_STATUS_REPO_REF"

# Bootstrap: validate URL before downloading utils.sh
case "$WAZUH_AGENT_STATUS_REPO_URL" in https://*) ;; *) echo "[ERROR] WAZUH_AGENT_STATUS_REPO_URL must use HTTPS" >&2; exit 1 ;; esac

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
enforce_https_url "$WAZUH_AGENT_STATUS_REPO_URL" "WAZUH_AGENT_STATUS_REPO_URL"
ensure_os "Linux"

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
# Default to 'wazuh' user/group if they exist, otherwise fallback to root
USER_EXISTS=$(id -u wazuh 2>/dev/null || echo "")
GROUP_EXISTS=$(getent group wazuh 2>/dev/null || echo "")

if [[ -n "$USER_EXISTS" ]]; then
    WAZUH_USER=${WAZUH_USER:-"wazuh"}
else
    WAZUH_USER=${WAZUH_USER:-"root"}
fi

if [[ -n "$GROUP_EXISTS" ]]; then
    WAZUH_GROUP=${WAZUH_GROUP:-"wazuh"}
else
    WAZUH_GROUP=${WAZUH_GROUP:-"root"}
fi

REAL_USER=$(get_real_user)
REAL_HOME=$(eval echo "~$REAL_USER")

SERVICE_FILE=${SERVICE_FILE:-"/etc/systemd/system/$SERVER_NAME.service"}
MIGRATION_MARKER="/usr/local/etc/$SERVER_NAME/.migrated_from_go"
DESKTOP_UNIT_FOLDER=${DESKTOP_UNIT_FOLDER:-"$REAL_HOME/.config/autostart"}
DESKTOP_APPS_FOLDER=${DESKTOP_APPS_FOLDER:-"$REAL_HOME/.local/share/applications"}
USER_ICON_FOLDER=${USER_ICON_FOLDER:-"$REAL_HOME/.local/share/icons"}
DESKTOP_UNIT_FILE=${DESKTOP_UNIT_FILE:-"$DESKTOP_UNIT_FOLDER/$CLIENT_NAME.desktop"}
DESKTOP_APP_FILE=${DESKTOP_APP_FILE:-"$DESKTOP_APPS_FOLDER/$CLIENT_NAME.desktop"}
APP_ID="wazuh-agent-status"
BASE_URL=${BASE_URL:-"https://github.com/ADORSYS-GIS/${SERVER_NAME}/releases/download/v${APP_VERSION}"}

# Sanity check for BASE_URL: Automatically correct GitHub tag page URLs to download URLs
if [[ "${BASE_URL}" == *"releases/tag/"* ]]; then
    warn_message "BASE_URL appears to point to a tag page instead of a release download: ${BASE_URL}"
    warn_message "Correcting BASE_URL to use 'download' path..."
    BASE_URL="${BASE_URL/releases\/tag/releases/download}"
    info_message "Corrected BASE_URL: ${BASE_URL}"
fi

ICON_URL="${BASE_URL}/icon.png"

SERVER_BIN_NAME="${SERVER_NAME}-${OS}-${ARCH}"
CLIENT_BIN_NAME="${CLIENT_NAME}-${OS}-${ARCH}"
SERVER_URL="${BASE_URL}/${SERVER_BIN_NAME}"
CLIENT_URL="${BASE_URL}/${CLIENT_BIN_NAME}"
CHECKSUM_URL="${BASE_URL}/checksums.sha256"

ADORSYS_UPDATE_SCRIPT_URL=${ADORSYS_UPDATE_SCRIPT_URL:-"$WAZUH_AGENT_STATUS_REPO_URL/scripts/linux/adorsys-update.sh"}
UPDATE_SCRIPT_PATH="$WAZUH_ACTIVE_RESPONSE_BIN_DIR/adorsys-update.sh"

# Legacy Go Cleanup
cleanup_legacy_system() {
    if [[ -f "$MIGRATION_MARKER" ]]; then
        info_message "Migration already completed. Skipping legacy cleanup."
        return 0
    fi

    print_step_header 0 "Legacy Go Cleanup"
    info_message "Detecting legacy Go components..."

    # 1. Stop and Disable the old service (if it exists)
    if command_exists systemctl; then
        if systemctl is-active --quiet "$SERVER_NAME" 2>/dev/null; then
            info_message "Stopping legacy service: $SERVER_NAME"
            maybe_sudo systemctl stop "$SERVER_NAME" || true
        fi
        if systemctl is-enabled --quiet "$SERVER_NAME" 2>/dev/null; then
            info_message "Disabling legacy service: $SERVER_NAME"
            maybe_sudo systemctl disable "$SERVER_NAME" || true
        fi
    fi

    # 2. Kill any lingering Go processes
    info_message "Killing lingering Go processes ($SERVER_NAME, $CLIENT_NAME)..."
    maybe_sudo killall "$SERVER_NAME" 2>/dev/null || true
    maybe_sudo killall "$CLIENT_NAME" 2>/dev/null || true

    # 3. Remove old desktop entries
    if [[ -f "$DESKTOP_UNIT_FILE" ]]; then
        info_message "Removing legacy desktop entry: $DESKTOP_UNIT_FILE"
        remove_file "$DESKTOP_UNIT_FILE"
    fi
    if [[ -f "$DESKTOP_APP_FILE" ]]; then
        info_message "Removing legacy desktop app entry: $DESKTOP_APP_FILE"
        remove_file "$DESKTOP_APP_FILE"
    fi

    info_message "Legacy cleanup complete."
    return 0
}


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
Environment=WAZUH_STATUS_LOG_FILE=/var/log/wazuh-agent-status/wazuh-agent-status.log

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
install_icon() {
    info_message "Creating user icon directory if it doesn't exist..."
    mkdir -p "$USER_ICON_FOLDER"

    info_message "Downloading application icon from release..."
    if ! download_and_verify_file "$ICON_URL" "$USER_ICON_FOLDER/$APP_ID.png" "icon.png" "application icon" "$CHECKSUM_URL"; then
        warn_message "Failed to download verified icon from $ICON_URL. Application will use a generic icon."
        return 0
    fi
    info_message "Icon installed to: $USER_ICON_FOLDER/$APP_ID.png"
    return 0
}

configure_logrotate() {
    local logrotate_file="/etc/logrotate.d/wazuh-agent-status"
    local log_dir="/var/log/wazuh-agent-status"
    local log_file_path="${log_dir}/wazuh-agent-status.log"

    info_message "Configuring logrotate for $log_file_path..."

    create_file "$logrotate_file" "$log_file_path {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 $WAZUH_USER $WAZUH_GROUP
}
"
    # Ensure correct permissions for the logrotate entry
    maybe_sudo chown root:root "$logrotate_file"
    maybe_sudo chmod 644 "$logrotate_file"

    success_message "Logrotate configuration created: $logrotate_file"
    return 0
}

create_desktop_unit_file() {
    info_message "Creating desktop directories if they don't exist..."
    mkdir -p "$DESKTOP_UNIT_FOLDER"
    mkdir -p "$DESKTOP_APPS_FOLDER"

    local desktop_content="[Desktop Entry]
Name=Wazuh Agent Monitor
GenericName=Wazuh Agent Status
Comment=Monitors the Wazuh agent status and provides a tray icon dashboard
Exec=$BIN_DIR/$CLIENT_NAME
Icon=$APP_ID
Terminal=false
Type=Application
StartupWMClass=$CLIENT_NAME
X-GNOME-Autostart-enabled=true
Categories=Utility;System;Monitoring;
"

    info_message "Creating desktop unit file for autostart..."
    create_file "$DESKTOP_UNIT_FILE" "$desktop_content"
    
    info_message "Creating desktop application entry..."
    create_file "$DESKTOP_APP_FILE" "$desktop_content"

    info_message "Desktop entries created successfully."
    return 0
}

# Startup Configurations
make_server_launch_at_startup() {
    create_service_file && reload_and_enable_service
    return 0
}

make_client_launch_at_startup() {
    install_icon
    create_desktop_unit_file
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

    if [[ -f "$DESKTOP_APP_FILE" ]]; then
        success_message "Desktop application file exists: $DESKTOP_APP_FILE."
    else
        error_exit "Desktop application file is missing: $DESKTOP_APP_FILE."
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
    download_and_verify_file "$ADORSYS_UPDATE_SCRIPT_URL" "$UPDATE_SCRIPT_PATH" "scripts/linux/adorsys-update.sh" "adorsys-update.sh script" "$WAZUH_AGENT_STATUS_REPO_URL/checksums.sha256" || warn_message "Failed to download adorsys-update.sh"
    maybe_sudo chmod 750 "$UPDATE_SCRIPT_PATH"

    # Update WAZUH_MANAGER value in adorsys-update.sh
    if [[ -n "${WAZUH_MANAGER:-}" ]]; then
        info_message "Updating WAZUH_MANAGER in adorsys-update.sh to $WAZUH_MANAGER"
        sed_inplace "s/^WAZUH_MANAGER=.*/WAZUH_MANAGER=\${WAZUH_MANAGER:-\"$WAZUH_MANAGER\"}/" "$UPDATE_SCRIPT_PATH"
    else
        warn_message "WAZUH_MANAGER variable not set. Skipping update in adorsys-update.sh."
    fi
else
    warn_message "$WAZUH_ACTIVE_RESPONSE_BIN_DIR does not exist, skipping."
fi

# Permissions and Ownership Configuration
print_step_header 6 "Permissions and Ownership Configuration"
setup_permissions_and_ownership "$WAZUH_USER" "$WAZUH_GROUP" "/var/ossec/bin/wazuh-control"

print_step_header 7 "Logrotate Configuration"
configure_logrotate

print_step_header 8 "Validating installation and configuration..."
validate_installation

# Create migration marker
maybe_sudo mkdir -p "$(dirname "$MIGRATION_MARKER")"
maybe_sudo touch "$MIGRATION_MARKER"
