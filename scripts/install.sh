#!/bin/sh

# Set shell options based on shell type
if [ -n "$BASH_VERSION" ]; then
    set -euo pipefail
else
    set -eu
fi

# Default log level and application details
APP_NAME=${APP_NAME:-"wazuh-agent-status"}
WOPS_VERSION=${WOPS_VERSION:-"0.1.2"}
WAZUH_USER=${WAZUH_USER:-"root"}

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

# Function to create the systemd service file
create_desktop_unit_file() {

    local DESKTOP_UNIT_FILE=${DESKTOP_UNIT_FILE:-"$HOME/.config/autostart/$APP_NAME.desktop"}
    
    if [ -f "$DESKTOP_UNIT_FILE" ]; then
        info_message "Service file $DESKTOP_UNIT_FILE already exists. Deleting..."
        sudo rm -f "$DESKTOP_UNIT_FILE"
        info_message "Old version of desktop unit file deleted successfully"
    fi

    echo "Creating desktop unit file at $SERVICE_FILE..."

    sudo bash -c "cat > $DESKTOP_UNIT_FILE" <<EOF
[Desktop Entry]
Name=Wazuh Agent Monitoring Tray Icon App
GenericName=A script that runs at Gnome startup
Comment=Runs the script in Exec path
Exec=sh -c "SUDO_ASKPASS=/usr/bin/ssh-askpass sudo -A $BIN_DIR/$APP_NAME >> $HOME/.wazuh-agent-status.log 2>&1"
Terminal=false
Type=Application
X-GNOME-Autostart-enabled=true


EOF

    info_message "Desktop unit file created."
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

# Construct binary name and URL for download
BIN_NAME="$APP_NAME-${OS}-${ARCH}"
BASE_URL="https://github.com/ADORSYS-GIS/$APP_NAME/releases/download/v$WOPS_VERSION"
URL="$BASE_URL/$BIN_NAME"

echo $URL

# Create a temporary directory and ensure it is cleaned up
TEMP_DIR=$(mktemp -d) || error_exit "Failed to create temporary directory"
trap 'rm -rf "$TEMP_DIR"' EXIT

# Step 1: Download the binary file
print_step 1 "Downloading $BIN_NAME from $URL..."
curl -SL --progress-bar -o "$TEMP_DIR/$BIN_NAME" "$URL" || error_exit "Failed to download $BIN_NAME"

# Step 2: Install the binary
print_step 2 "Installing binary to $BIN_DIR..."
maybe_sudo mv "$TEMP_DIR/$BIN_NAME" "$BIN_DIR/$APP_NAME" || error_exit "Failed to move binary to $BIN_DIR"
maybe_sudo chmod 111 "$BIN_DIR/$APP_NAME" || error_exit "Failed to set executable permissions on the binary"

# Step 3: Run the binary as a service
print_step 3 "Starting service creation process..."
sudo apt install -y ssh-askpass
sudo apt install -y ssh-askpass-gnome
create_desktop_unit_file
info_message "Desktop unit creation and setup complete."


success_message "Installation and configuration complete! You can now use '$APP_NAME' from your terminal."
info_message "Run \n\n\t${GREEN}${BOLD}$APP_NAME ${NORMAL}\n\n to start configuring. If you don't have sudo on your machine, you can run the command without sudo."
