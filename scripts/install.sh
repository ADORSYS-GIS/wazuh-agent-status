#!/bin/sh

# Set shell options based on shell type
if [ -n "$BASH_VERSION" ]; then
    set -euo pipefail
else
    set -eu
fi

# Default log level and application details
LOG_LEVEL=${LOG_LEVEL:-INFO}
APP_NAME=${APP_NAME:-"wazuh-agent-status"}
WOPS_VERSION=${WOPS_VERSION:-"0.2.1"}
OSSEC_CONF_PATH=${OSSEC_CONF_PATH:-"/var/ossec/etc/ossec.conf"}
USER="root"
GROUP="wazuh"

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

# Determine the OS and architecture
case "$(uname)" in
    "Linux") OS="unknown-linux-gnu"; BIN_DIR="$HOME/.local/bin" ;;
    "Darwin") OS="apple-darwin"; BIN_DIR="/usr/local/bin" ;;
    *) error_exit "Unsupported operating system: $(uname)" ;;
esac

ARCH=$(uname -m)
case "$ARCH" in
    "x86_64") ARCH="x86_64" ;;
    "arm64"|"aarch64") ARCH="aarch64" ;;
    *) error_exit "Unsupported architecture: $ARCH" ;;
esac

# Construct binary name and URL for download
BIN_NAME="$APP_NAME-${ARCH}-${OS}"
BASE_URL="https://github.com/ADORSYS-GIS/wazuh-agent-status/releases/download/v$WOPS_VERSION"
URL="$BASE_URL/$BIN_NAME"

# Create a temporary directory and ensure it is cleaned up
TEMP_DIR=$(mktemp -d) || error_exit "Failed to create temporary directory"
trap 'rm -rf "$TEMP_DIR"' EXIT

# Step 1: Download the binary file
print_step 1 "Downloading $BIN_NAME from $URL..."
curl -SL --progress-bar -o "$TEMP_DIR/$BIN_NAME" "$URL" || error_exit "Failed to download $BIN_NAME"

# Step 2: Install the binary
print_step 2 "Installing binary to $BIN_DIR..."
maybe_sudo mkdir -p "$BIN_DIR" || error_exit "Failed to create directory $BIN_DIR"
maybe_sudo mv "$TEMP_DIR/$BIN_NAME" "$BIN_DIR/$APP_NAME" || error_exit "Failed to move binary to $BIN_DIR"
maybe_sudo chmod 750 "$BIN_DIR/$APP_NAME" || error_exit "Failed to set executable permissions on the binary"

# Step 3: Update shell configuration
print_step 3 "Updating shell configuration..."

# Determine the appropriate shell configuration file
CURRENT_SHELL=$(echo $SHELL)

case "$CURRENT_SHELL" in
    *zsh)
        SHELL_RC="$HOME/.zshrc"
        ;;
    *bash)
        SHELL_RC="$HOME/.bashrc"
        ;;
    *)
        SHELL_RC="$HOME/.bashrc"
        ;;
esac

# If not yet present, add binary directory to PATH and set RUST_LOG environment variable
if ! grep -q "export PATH=\"$BIN_DIR:\$PATH\"" "$SHELL_RC"; then
    info_message "Adding $BIN_DIR to PATH in $SHELL_RC..."
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$SHELL_RC"
    info_message "Updated PATH in $SHELL_RC"
fi

if [ -f "$SHELL_RC" ]; then
    warn_message "Please run 'source $SHELL_RC' or open a new terminal to apply changes."
else
    warn_message "No configuration file found or changes might not apply."
fi
