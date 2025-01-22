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
WAZUH_USER=${WAZUH_USER:-"root"}

SERVICE_FILE=${SERVICE_FILE:-"/etc/systemd/system/$SERVER_NAME.service"}
SERVER_LAUNCH_AGENT_FILE=${SERVER_LAUNCH_AGENT_FILE:-"/Library/LaunchDaemons/com.adorsys.$SERVER_NAME.plist"}
CLIENT_LAUNCH_AGENT_FILE=${CLIENT_LAUNCH_AGENT_FILE:-"/Library/LaunchAgents/com.adorsys.$CLIENT_NAME.plist"}
DESKTOP_UNIT_FOLDER=${DESKTOP_UNIT_FOLDER:-"$HOME/.config/autostart"}
DESKTOP_UNIT_FILE=${DESKTOP_UNIT_FILE:-"$DESKTOP_UNIT_FOLDER/$CLIENT_NAME.desktop"}

PROFILE=${PROFILE:-"user"}
APP_VERSION=${APP_VERSION:-"0.2.5"}

# Assign app version based on profile
case "$PROFILE" in
    "admin") WAS_VERSION="$APP_VERSION" ;;
    *) WAS_VERSION="$APP_VERSION-user" ;;
esac


# OS and Architecture Detection
case "$(uname)" in
    Linux) OS="linux"; BIN_DIR="/usr/local/bin" ;;
    Darwin) OS="darwin"; BIN_DIR="/usr/local/bin" ;;
    *) error_exit "Unsupported operating system: $(uname)" ;;
esac

ARCH=$(uname -m)
case "$ARCH" in
    x86_64) ARCH="amd64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *) error_exit "Unsupported architecture: $ARCH" ;;
esac

# Text Formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[1;34m'
BOLD='\033[1m'
NORMAL='\033[0m'

# Logging Utilities
log() { echo -e "$(date +"%Y-%m-%d %H:%M:%S") $1 $2"; }
info_message() { log "${BLUE}${BOLD}[INFO]${NORMAL}" "$*"; }
warn_message() { log "${YELLOW}${BOLD}[WARNING]${NORMAL}" "$*"; }
error_message() { log "${RED}${BOLD}[ERROR]${NORMAL}" "$*"; }
success_message() { log "${GREEN}${BOLD}[SUCCESS]${NORMAL}" "$*"; }
print_step_header() { echo -e "\n${BOLD}===== STEP $1: $2 =====${NORMAL}\n"; }

# Error Handler
error_exit() {
    error_message "$1"
    exit 1
}

# Command Existence Check
command_exists() { command -v "$1" >/dev/null 2>&1; }

# Execute with Root Privileges
maybe_sudo() {
    if [ "$(id -u)" -ne 0 ]; then
        command_exists sudo && sudo "$@" || error_exit "This script requires root privileges. Run as root or use sudo."
    else
        "$@"
    fi
}

# General Utility Functions
create_file() {
    local filepath="$1"
    local content="$2"
    maybe_sudo bash -c "cat > \"$filepath\" <<EOF
$content
EOF"
    info_message "Created file: $filepath"
}

remove_file() {
    local filepath="$1"
    if [ -f "$filepath" ]; then
        info_message "Removing file: $filepath"
        maybe_sudo rm -f "$filepath"
    fi
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

[Install]
WantedBy=multi-user.target
"
    info_message "Systemd service file created: $SERVICE_FILE"
}

reload_and_enable_service() {
    info_message "Reloading systemd daemon..."
    maybe_sudo systemctl daemon-reload
    
    info_message "Enabling service to start at boot..."
    maybe_sudo systemctl enable "$SERVER_NAME"
    
    info_message "Starting the service..."
    maybe_sudo systemctl start "$SERVER_NAME"
    
    info_message "Systemd service enabled and started."
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
    info_message "Unloading previous plist file (if any)..."
    maybe_sudo launchctl unload "$filepath" 2>/dev/null || true
    
    info_message "Loading new plist file..."
    maybe_sudo launchctl load -w "$filepath"
    
    info_message "macOS Launchd plist file created and loaded: $filepath"
}

# Startup Configurations
make_server_launch_at_startup() {
    case "$OS" in
        linux) create_service_file && reload_and_enable_service ;;
        darwin) create_launchd_plist_file "$SERVER_NAME" "$SERVER_LAUNCH_AGENT_FILE" ;;
    esac
}

make_client_launch_at_startup() {
    case "$OS" in
        linux) create_desktop_unit_file ;;
        darwin) create_launchd_plist_file "$CLIENT_NAME" "$CLIENT_LAUNCH_AGENT_FILE" ;;
    esac
}

validate_installation() {
    # Validate binaries
    if [ -x "$BIN_DIR/$SERVER_NAME" ]; then
        success_message "Server binary exists and is executable: $BIN_DIR/$SERVER_NAME."
    else
        error_exit "Server binary is missing or not executable: $BIN_DIR/$SERVER_NAME."
    fi

    if [ -x "$BIN_DIR/$CLIENT_NAME" ]; then
        success_message "Client binary exists and is executable: $BIN_DIR/$CLIENT_NAME."
    else
        error_exit "Client binary is missing or not executable: $BIN_DIR/$CLIENT_NAME."
    fi

    # Validate service files
    if [ "$OS" = "linux" ]; then
        if [ -f "$SERVICE_FILE" ]; then
            success_message "Systemd service file exists: $SERVICE_FILE."
        else
            error_exit "Systemd service file is missing: $SERVICE_FILE."
        fi

        systemctl is-enabled "$SERVER_NAME" >/dev/null 2>&1 && \
            success_message "Systemd service is enabled: $SERVER_NAME." || \
            error_exit "Systemd service is not enabled: $SERVER_NAME."
            
        if [ -f "$DESKTOP_UNIT_FILE" ]; then
            success_message "Desktop autostart file exists: $DESKTOP_UNIT_FILE."
        else
            error_exit "Desktop autostart file is missing: $DESKTOP_UNIT_FILE."
        fi
        
    elif [ "$OS" = "darwin" ]; then
        if [ -f "$SERVER_LAUNCH_AGENT_FILE" ]; then
            success_message "macOS Launchd server plist exists: $SERVER_LAUNCH_AGENT_FILE."
        else
            error_exit "macOS Launchd server plist is missing: $SERVER_LAUNCH_AGENT_FILE."
        fi

        if [ -f "$CLIENT_LAUNCH_AGENT_FILE" ]; then
            success_message "macOS Launchd client plist exists: $CLIENT_LAUNCH_AGENT_FILE."
        else
            error_exit "macOS Launchd client plist is missing: $CLIENT_LAUNCH_AGENT_FILE."
        fi
    fi

    case "$OS" in
        linux) success_message "Installation complete! Restart your system to apply changes for the wazuh agent status." ;;
        darwin) success_message "Installation complete!" ;;
    esac
}

# Installation Process
TEMP_DIR=$(mktemp -d) || error_exit "Failed to create temporary directory"
trap 'rm -rf "$TEMP_DIR"' EXIT

SERVER_BIN_NAME="$SERVER_NAME-$OS-$ARCH"
CLIENT_BIN_NAME="$CLIENT_NAME-$OS-$ARCH"
BASE_URL="https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/tag/v$WAS_VERSION"
SERVER_URL="$BASE_URL/$SERVER_BIN_NAME"
CLIENT_URL="$BASE_URL/$CLIENT_BIN_NAME"

echo "$PROFILE"
echo "$BASE_URL"

print_step_header 1 "Binaries Download"
info_message "Downloading server binary from $SERVER_URL..."
curl -SL -o "$TEMP_DIR/$SERVER_BIN_NAME" "$SERVER_URL" || error_exit "Failed to download $SERVER_BIN_NAME"
info_message "Downloading client binary $CLIENT_URL..."
curl -SL -o "$TEMP_DIR/$CLIENT_BIN_NAME" "$CLIENT_URL" || error_exit "Failed to download $CLIENT_BIN_NAME"
success_message "Binaries downloaded successfully."

print_step_header 2 "Binaries Installation"
info_message "Installing server binary to $BIN_DIR..."
maybe_sudo mv "$TEMP_DIR/$SERVER_BIN_NAME" "$BIN_DIR/$SERVER_NAME"
maybe_sudo chmod +x "$BIN_DIR/$SERVER_NAME"
info_message "Installing client binary to $BIN_DIR..."
maybe_sudo mv "$TEMP_DIR/$CLIENT_BIN_NAME" "$BIN_DIR/$CLIENT_NAME"
maybe_sudo chmod +x "$BIN_DIR/$CLIENT_NAME"
success_message "Binaries installed successfully."

print_step_header 3 "Server Service Configuration"
make_server_launch_at_startup

print_step_header 4 "Client Service Configuration"
make_client_launch_at_startup

print_step_header 5 "Validating installation and configuration..."
validate_installation