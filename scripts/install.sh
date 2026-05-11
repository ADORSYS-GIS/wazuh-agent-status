#!/bin/sh

# Unified Installation Script for Wazuh Agent Status
# Detects OS and dispatches to platform-specific scripts

set -eu

# Detect OS
OS_TYPE=$(uname -s)

case "$OS_TYPE" in
    Linux*)
        SCRIPT_PATH="linux/install.sh"
        ;;
    Darwin*)
        SCRIPT_PATH="macos/install.sh"
        ;;
    *)
        echo "Error: Unsupported operating system: $OS_TYPE"
        exit 1
        ;;
esac

# Common configuration
WAZUH_AGENT_STATUS_REPO_REF=${WAZUH_AGENT_STATUS_REPO_REF:-"user-main"}
WAZUH_AGENT_STATUS_REPO_URL="https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/$WAZUH_AGENT_STATUS_REPO_REF"

# Download and run the specific script
TMP_DIR=$(mktemp -d)
# Ensure we cleanup on exit, but we don't want to cleanup if we are sourcing
# so we run it in a subshell or just download and execute

# Since we want to pass the environment variables, we download and execute with sudo -E if needed
# but the caller probably already used sudo

install_script_url="${WAZUH_AGENT_STATUS_REPO_URL}/scripts/${SCRIPT_PATH}"

echo "Detected OS: $OS_TYPE"
echo "Fetching installation script from: $install_script_url"

if ! curl -fsSL "$install_script_url" -o "$TMP_DIR/install.sh"; then
    echo "Error: Failed to download $SCRIPT_PATH from $install_script_url"
    exit 1
fi

# Run the actual script
# We use -E to preserve environment variables like APP_VERSION, WAZUH_MANAGER, etc.
sh "$TMP_DIR/install.sh" "$@"

# Cleanup
rm -rf "$TMP_DIR"
