#!/bin/sh

# Set shell options based on shell type
if [ -n "$BASH_VERSION" ]; then
    set -euo pipefail
else
    set -eu
fi

# Default log level and application details
SERVER_NAME=${SERVER_NAME:-"wazuh-agent-status"}
CLIENT_NAME=${CLIENT_NAME:-"wazuh-agent-status-client"}
WOPS_VERSION=${WOPS_VERSION:-"0.2.1"}
WAZUH_USER=${WAZUH_USER:-"root"}
SERVICE_FILE=${SERVICE_FILE:-"/etc/systemd/system/$SERVER_NAME.service"}

# Define text formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[1;34m'
BOLD='\033[1m'
NORMAL='\033[0m'

# Function for logging with timestamp
log() {
    local LEVEL="$1"
    shift
    local MESSAGE="$*"
    local TIMESTAMP
    TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
    echo -e "${TIMESTAMP} ${LEVEL} ${MESSAGE}"
}

# Logging helpers
info_message() {
    log "${BLUE}${BOLD}[INFO]${NORMAL}" "$*"
}

warn_message() {
    log "${YELLOW}${BOLD}[WARNING]${NORMAL}" "$*"
}

error_message() {
    log "${RED}${BOLD}[ERROR]${NORMAL}" "$*"
}

success_message() {
    log "${GREEN}${BOLD}[SUCCESS]${NORMAL}" "$*"
}

print_step() {
    log "${BLUE}${BOLD}[STEP]${NORMAL}" "$1: $2"
}

# Exit script with an error message
error_exit() {
    error_message "$1"
    exit 1
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Ensure root privileges, either directly or through sudo
maybe_sudo() {
    if [ "$(id -u)" -ne 0 ]; then
        if command_exists sudo; then
            sudo "$@"
        else
            error_message "This script requires root privileges. Please run with sudo or as root."
            exit 1
        fi
    else
        "$@"
    fi
}

# Function to reload systemd and enable the service
remove_service() {
    info_message "Stopping $SERVER_NAME service..."
    sudo systemctl stop $SERVER_NAME
    
    info_message "Deleting $SERVICE_FILE file..."
    sudo rm $SERVICE_FILE
    
    info_message "Reloading systemd daemon..."
    sudo systemctl daemon-reload
}
# Function to create the systemd service file
create_service_file() {

    if [ -f "$SERVICE_FILE" ]; then
        info_message "Service file $SERVICE_FILE already exists. Deleting..."
        remove_service
        info_message "Old version of service file deleted successfully"
    fi

    echo "Creating service file at $SERVICE_FILE..."

    sudo bash -c "cat > $SERVICE_FILE" <<EOF
[Unit]
Description=Wazuh Agent Status daemon
After=network.target

[Service]
ExecStart=$BIN_DIR/$SERVER_NAME
Restart=always
User=$WAZUH_USER

[Install]
WantedBy=multi-user.target
EOF

    info_message "Service file created."
}

# Function to reload systemd and enable the service
reload_and_enable_service() {
    info_message "Reloading systemd daemon..."
    sudo systemctl daemon-reload

    info_message "Enabling service to start on boot..."
    sudo systemctl enable $SERVER_NAME.service

    info_message "Starting the service..."
    sudo systemctl start $SERVER_NAME.service
}

# Function to create the systemd service file
create_desktop_unit_file() {

    local DESKTOP_UNIT_FOLDER=${DESKTOP_UNIT_FOLDER:-"$HOME/.config/autostart"}
    local DESKTOP_UNIT_FILE=${DESKTOP_UNIT_FILE:-"$HOME/.config/autostart/$CLIENT_NAME.desktop"}
    local COMMAND="$CLIENT_NAME"
    
    # Check if the parent directory exists
    if [ ! -d "$DESKTOP_UNIT_FOLDER" ]; then
        info_message "Parent directory does not exist. Creating it now..."
        mkdir -p "$DESKTOP_UNIT_FOLDER"
        info_message "Parent directory created."
    else
        info_message "Parent directory already exists."
    fi
    
    # Check if the desktop unit file already exists and delete it if it does
    if [ -f "$DESKTOP_UNIT_FILE" ]; then
        info_message "Service file $DESKTOP_UNIT_FILE already exists. Deleting..."
        sudo rm -f "$DESKTOP_UNIT_FILE"
        info_message "Old version of desktop unit file deleted successfully"
    fi
    
    

    echo "Creating desktop unit file at $DESKTOP_UNIT_FILE..."

    sudo bash -c "cat > $DESKTOP_UNIT_FILE" <<EOF
[Desktop Entry]
Name=Wazuh Agent Monitoring Tray Icon App
GenericName=A script that runs at Gnome startup
Comment=Runs the script in Exec path
Exec=$BIN_DIR/$CLIENT_NAME
Terminal=false
Type=Application
X-GNOME-Autostart-enabled=true


EOF

    info_message "Desktop unit file created."
    
}

make_app_launch_at_startup() {

    if [ "$(uname)" == "Linux" ]; then
        info_message "Creating tray icon automatic launch agent..."
        create_desktop_unit_file
        info_message "Tray icon automatic launch agent created"
    fi

}

# Function to check if the binary exists
check_binary_exists() {
    if [ ! -f "$BIN_DIR" ]; then
        warn_message "Binary $BIN_DIR does not exist. Exiting."
        exit 1
    fi
}

# Determine the OS and architecture
case "$(uname)" in
    "Linux") OS="linux"; BIN_DIR="/usr/local/bin" ;;
    "Darwin") OS="darwin"; BIN_DIR="/usr/local/bin" ;;
    *) error_exit "Unsupported operating system: $(uname)" ;;
esac

ARCH=$(uname -m)
case "$ARCH" in
    "x86_64") ARCH="amd64" ;;
    "arm64"|"aarch64") ARCH="arm64" ;;
    *) error_exit "Unsupported architecture: $ARCH" ;;
esac

#https://github.com/ADORSYS-GIS/wazuh-agent-status/releases/download/v0.1.2/wazuh-agent-status-darwin-arm64

# Construct the server binary name and URL for download
SERVER_BIN_NAME="$SERVER_NAME-${OS}-${ARCH}"
SERVER_BASE_URL="https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/download/v$WOPS_VERSION"
SERVER_URL="$SERVER_BASE_URL/$SERVER_BIN_NAME"

# Construct the client binary name and URL for download
CLIENT_BIN_NAME="$CLIENT_NAME-${OS}-${ARCH}"
CLIENT_BASE_URL="https://github.com/ADORSYS-GIS/$SERVER_NAME/releases/download/v$WOPS_VERSION"
CLIENT_URL="$CLIENT_BASE_URL/$CLIENT_BIN_NAME"

echo $CLIENT_URL
echo $SERVER_URL

# Create a temporary directory and ensure it is cleaned up
TEMP_DIR=$(mktemp -d) || error_exit "Failed to create temporary directory"
trap 'rm -rf "$TEMP_DIR"' EXIT

# Step 1: Download the binaries
print_step 1 "Downloading binaries..."
info_message "Downloading $SERVER_BIN_NAME from $SERVER_URL..."
curl -SL --progress-bar -o "$TEMP_DIR/$SERVER_BIN_NAME" "$SERVER_URL" || error_exit "Failed to download $SERVER_BIN_NAME"
info_message "Downloading $CLIENT_BIN_NAME from $CLIENT_URL..."
curl -SL --progress-bar -o "$TEMP_DIR/$CLIENT_BIN_NAME" "$CLIENT_URL" || error_exit "Failed to download $CLIENT_BIN_NAME"

# Step 2: Install the binaries
print_step 2 "Installing binaries to $BIN_DIR..."
maybe_sudo mv "$TEMP_DIR/$SERVER_BIN_NAME" "$BIN_DIR/$SERVER_NAME" || error_exit "Failed to move binary to $BIN_DIR"
maybe_sudo chmod 111 "$BIN_DIR/$SERVER_NAME" || error_exit "Failed to set executable permissions on the binary"

maybe_sudo mv "$TEMP_DIR/$CLIENT_BIN_NAME" "$BIN_DIR/$CLIENT_NAME" || error_exit "Failed to move binary to $BIN_DIR"
maybe_sudo chmod 111 "$BIN_DIR/$CLIENT_NAME" || error_exit "Failed to set executable permissions on the binary"

# Step 3: Run the binary as a service
print_step 3 "Starting service creation..."
create_service_file
reload_and_enable_service
info_message "Service creation and setup complete."

# Step 4: Run the binary as a service
print_step 4 "Starting desktop unit creation..."
make_app_launch_at_startup

success_message "Installation and configuration complete! You can now use '$SERVER_NAME' on your host"
warn_message "You need to reboot your for the changes to take effect"
