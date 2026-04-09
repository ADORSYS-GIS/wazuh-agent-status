#!/usr/bin/env bash

# Centralized Utility Functions for Wazuh-Agent-Status Scripts
# Designed to be downloaded and sourced via a bootstrap mechanism

# Define text formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[1;34m'
BOLD='\033[1m'
NORMAL='\033[0m'

# Function for logging with timestamp
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp
    timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    echo -e "${timestamp} ${level} ${message}"
    return 0
}

# Logging helpers
info_message() {
    log "${BLUE}${BOLD}[INFO]${NORMAL}" "$*"
    return 0
}

warn_message() {
    log "${YELLOW}${BOLD}[WARNING]${NORMAL}" "$*"
    return 0
}

error_message() {
    log "${RED}${BOLD}[ERROR]${NORMAL}" "$*"
    return 0
}

error_exit() {
    error_message "$*"
    exit 1
}

success_message() {
    log "${GREEN}${BOLD}[SUCCESS]${NORMAL}" "$*"
    return 0
}

print_step_header() {
    echo -e "\n${BOLD}===== STEP $1: $2 =====${NORMAL}\n"
    return 0
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
    return 0
}

# Detect system architecture
detect_architecture() {
    local arch
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            echo "amd64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        *)
            error_message "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Check if sudo is available or if the script is run as root
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

# Portable sed inplace
sed_inplace() {
    if [[ "$(uname -s)" == "Darwin" ]]; then
        maybe_sudo sed -i '' "$@" 2>/dev/null || true
    else
        maybe_sudo sed -i "$@" 2>/dev/null || true
    fi
}

remove_file() {
    local filepath="$1"
    if [ -f "$filepath" ]; then
        info_message "Removing file: $filepath"
        maybe_sudo rm -f "$filepath"
    fi
}

calculate_sha256() {
    local file="$1"
    if command_exists sha256sum; then
        sha256sum "$file" | awk '{print $1}'
    elif command_exists shasum; then
        shasum -a 256 "$file" | awk '{print $1}'
    else
        error_message "No SHA256 tool available (sha256sum or shasum required)"
        return 1
    fi
    return 0
}

verify_checksum() {
    local file="$1"
    local expected="$2"
    local actual
    actual=$(calculate_sha256 "$file")

    if [[ "$actual" != "$expected" ]]; then
        error_message "Checksum verification FAILED for $file!"
        error_message "  Expected: $expected"
        error_message "  Got:      $actual"
        return 1
    fi
    return 0
}

download_file() {
    local url="$1"
    local dest="$2"
    local description="${3:-file}"
    local max_retries="${4:-3}"
    local retry_count=0

    info_message "Downloading $description..."

    if [[ -z "$url" ]] || [[ -z "$dest" ]]; then
        error_message "Usage: download_file <url> <destination> [description] [max_retries]"
        return 1
    fi

    maybe_sudo mkdir -p "$(dirname "$dest")"

    while [[ "$retry_count" -lt "$max_retries" ]]; do
        if command_exists curl; then
            if curl -fsSL --retry 3 --retry-delay 2 "$url" | maybe_sudo tee "$dest" > /dev/null; then
                success_message "$description downloaded successfully"
                return 0
            fi
        elif command_exists wget; then
            if wget -q --tries=3 --wait=2 -O - "$url" | maybe_sudo tee "$dest" > /dev/null; then
                success_message "$description downloaded successfully"
                return 0
            fi
        else
            error_message "Neither curl nor wget is available"
            return 1
        fi
        retry_count=$((retry_count + 1))
        warn_message "Download failed, retrying (${retry_count}/${max_retries})..."
        sleep 2
    done

    error_message "Failed to download $description from $url after ${max_retries} attempts"
    return 1
}

download_and_verify_file() {
    local url="$1"
    local dest="$2"
    local pattern="$3"
    local name="${4:-Unknown file}"
    local checksum_url="${5:-${CHECKSUMS_URL:-}}"
    local checksum_file="${6:-${CHECKSUMS_FILE:-}}"

    if ! download_file "$url" "$dest" "$name"; then
        error_exit "Failed to download $name from $url"
    fi

    if [[ -n "$checksum_url" ]]; then
        local temp_checksum_file
        temp_checksum_file=$(mktemp)
        if ! download_file "$checksum_url" "$temp_checksum_file" "checksum file"; then
            error_exit "Failed to download external checksum file from $checksum_url"
        fi
        checksum_file="$temp_checksum_file"
    fi

    if [[ -f "$checksum_file" ]]; then
        local expected
        expected=$(grep "$pattern" "$checksum_file" | awk '{print $1}')

        if [[ -n "$expected" ]]; then
            if ! verify_checksum "$dest" "$expected"; then
                error_exit "$name checksum verification failed"
            fi
            info_message "$name checksum verification passed."
        else
            error_exit "No checksum found for $name in $checksum_file using pattern $pattern"
        fi

        # Cleanup temporary checksum file if it was downloaded from a URL
        if [[ -n "$checksum_url" ]] && [[ -f "$checksum_file" ]]; then
            rm -f "$checksum_file"
        fi
    else
        error_exit "Checksum file not found at $checksum_file, cannot verify $name"
    fi

    success_message "$name downloaded and verified successfully."
    return 0
}

# Cleanup function (can be overridden by caller)
cleanup() {
    info_message "Cleaning up temporary files..."
    if [ -n "${TMP_DIR:-}" ] && [ -d "$TMP_DIR" ]; then
        rm -rf "$TMP_DIR"
    fi
}

# Create file
create_file() {
    local filepath="$1"
    local content="$2"
    maybe_sudo bash -c "cat > \"$filepath\" <<EOF
$content
EOF"
    info_message "Created file: $filepath"
}
