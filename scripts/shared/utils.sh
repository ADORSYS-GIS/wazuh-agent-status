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
    printf "%s %b %s\n" "${timestamp}" "${level}" "${message}"
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
    local step_number="$1"
    local step_name="$2"
    printf "\n%b===== STEP %s: %s =====%b\n\n" "${BOLD}" "${step_number}" "${step_name}" "${NORMAL}"
    return 0
}

# Check if a command exists
command_exists() {
    local command="$1"
    command -v "$command" >/dev/null 2>&1
    return $?
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
            error_exit "Unsupported architecture: $arch"
            ;;
    esac
    return 0
}

# Check if sudo is available or if the script is run as root
maybe_sudo() {
    if [[ "$(id -u)" -ne 0 ]]; then
        if command_exists sudo; then
            sudo "$@"
        else
            error_exit "This script requires root privileges. Please run with sudo or as root."
        fi
    else
        "$@"
        return $?
    fi
    return 0
}

remove_file() {
    local filepath="$1"
    if [[ -f "$filepath" ]]; then
        info_message "Removing file: $filepath"
        maybe_sudo rm -f "$filepath"
        return $?
    fi
    return 0
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
            # If running as root, we can use -o directly. Otherwise, we might need sudo tee.
            if [[ "$(id -u)" -eq 0 ]]; then
                if curl -fsSL --retry 3 --retry-delay 2 "$url" -o "$dest"; then
                    success_message "$description downloaded successfully"
                    return 0
                fi
            else
                if curl -fsSL --retry 3 --retry-delay 2 "$url" | maybe_sudo tee "$dest" > /dev/null; then
                    success_message "$description downloaded successfully"
                    return 0
                fi
            fi
        elif command_exists wget; then
            if [[ "$(id -u)" -eq 0 ]]; then
                if wget -q --tries=3 --wait=2 -O "$dest" "$url"; then
                    success_message "$description downloaded successfully"
                    return 0
                fi
            else
                if wget -q --tries=3 --wait=2 -O - "$url" | maybe_sudo tee "$dest" > /dev/null; then
                    success_message "$description downloaded successfully"
                    return 0
                fi
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
    local url="${1}"
    local dest="${2}"
    local pattern="${3}"
    local name="${4:-Unknown file}"
    # Expected checksum file format: "sha256  filename" or "sha256 filename"
    local checksum_url="${5:-${CHECKSUMS_URL:-}}"
    local checksum_file="${6:-${CHECKSUMS_FILE:-}}"

    if ! download_file "${url}" "${dest}" "${name}"; then
        error_exit "Failed to download ${name} from ${url}"
    fi

    # Handle external checksum file download if a URL is provided
    if [[ -n "${checksum_url}" ]]; then
        local temp_checksum_file
        temp_checksum_file=$(mktemp)
        if ! download_file "${checksum_url}" "${temp_checksum_file}" "checksum file"; then
            error_exit "Failed to download external checksum file from ${checksum_url}"
        fi
        checksum_file="${temp_checksum_file}"
    fi

    # Verify checksum if a checksum file is available
    if [[ -f "${checksum_file}" ]]; then
        local expected
        # Use anchored grep for exact filename matching to avoid partial matches
        # Format: HASH  FILENAME (with one or more spaces)
        expected=$(grep -E "[[:space:]]+${pattern}$" "${checksum_file}" | awk '{print $1}' | head -n 1)

        if [[ -n "${expected}" ]]; then
            # Validate that the extracted checksum follows SHA256 hexadecimal format
            if ! [[ "${expected}" =~ ^[0-9a-fA-F]{64}$ ]]; then
                error_message "Detected invalid checksum format for ${name}: ${expected}"
                error_message "Problematic line in ${checksum_file}:"
                grep -E "[[:space:]]+${pattern}$" "${checksum_file}" | head -n 1
                error_exit "Invalid checksum entry in ${checksum_file}"
            fi

            if ! verify_checksum "${dest}" "${expected}"; then
                error_exit "${name} checksum verification FAILED"
            fi
            info_message "${name} checksum verification passed."
        else
            error_message "No checksum found for ${name} in ${checksum_file} with pattern '${pattern}'"
            error_message "First 10 lines of the checksum file for debugging:"
            head -n 10 "${checksum_file}"
            error_exit "Checksum lookup failed for ${name}"
        fi

        # Clean up temporary checksum files
        if [[ -n "${checksum_url}" ]] && [[ -f "${checksum_file}" ]]; then
            rm -f "${checksum_file}"
        fi
    else
        error_exit "Checksum file not found at ${checksum_file}; cannot verify ${name}"
    fi

    success_message "${name} downloaded and verified successfully."
    return 0
}

# Cleanup function (can be overridden by caller)
cleanup() {
    info_message "Cleaning up temporary files..."
    if [[ -n "${TMP_DIR:-}" ]] && [[ -d "${TMP_DIR}" ]]; then
        rm -rf "${TMP_DIR}"
        return $?
    fi
    return 0
}

# Create file
create_file() {
    local filepath="$1"
    local content="$2"
    maybe_sudo bash -c "cat > \"$filepath\" <<EOF
$content
EOF"
    info_message "Created file: $filepath"
    return 0
}

# Cross-platform sed -i wrapper
sed_inplace() {
    local expr="$1"
    local file="$2"

    if [[ -z "${file}" ]]; then
        error_message "sed_inplace: file argument is empty"
        return 1
    fi

    if [[ "$(uname -s)" == "Darwin" ]]; then
        # BSD sed (macOS) is finicky with -i. The most compatible way is -i.bak then rm.
        maybe_sudo sed -i.bak -e "${expr}" "${file}"
        maybe_sudo rm -f "${file}.bak"
    else
        # GNU sed (Linux)
        maybe_sudo sed -i -e "${expr}" "${file}"
    fi
}

# Runs a shell function with root privileges by injecting its definition
maybe_sudo_fn() {
    local fn="$1"; shift
    if [[ "$(id -u)" -ne 0 ]]; then
        command_exists sudo || error_exit "This script requires root privileges. Run as root or use sudo."
        sudo /usr/bin/env bash -c "$(declare -f "$fn"); $fn \"\$@\"" -- "$@"
    else
        "$fn" "$@"
    fi
}

# Detect the real user who invoked the script (even via sudo)
get_real_user() {
    # If SUDO_USER is set, trust it
    if [[ -n "${SUDO_USER:-}" ]]; then
        echo "$SUDO_USER"
        return
    fi

    # Check LOGNAME or USER if they are not root
    if [[ -n "${LOGNAME:-}" ]] && [[ "$LOGNAME" != "root" ]]; then
        echo "$LOGNAME"
        return
    fi
    if [[ -n "${USER:-}" ]] && [[ "$USER" != "root" ]]; then
        echo "$USER"
        return
    fi

    # Fallback for Linux using process tree
    if [[ "$(uname -s)" == "Linux" ]]; then
        local pid=$$
        while [ "$pid" -ne 1 ] && [ -n "$pid" ]; do
            pid=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ')
            [ -z "$pid" ] && break
            local user
            user=$(ps -o user= -p "$pid" 2>/dev/null | tr -d ' ')
            if [ -n "$user" ] && [ "$user" != "root" ]; then
                echo "$user"
                return
            fi
        done
    fi

    # Fallback for macOS using stat on the tty
    if [[ "$(uname -s)" == "Darwin" ]]; then
        local tty_user
        tty_user=$(stat -f "%Su" /dev/console 2>/dev/null)
        if [[ -n "$tty_user" ]] && [[ "$tty_user" != "root" ]]; then
            echo "$tty_user"
            return
        fi
    fi

    # Last resort: current user (might be root)
    id -un
}

# Grant passwordless sudo for wazuh-control to the wazuh user
setup_sudoers() {
    local wazuh_user="${1}"
    local wazuh_control_path="${2}"
    local sudoers_file="/etc/sudoers.d/wazuh-agent-status"

    # Only configure sudoers if the user is not root
    if [[ "${wazuh_user}" != "root" ]]; then
        info_message "Configuring sudoers for ${wazuh_user} to allow passwordless ${wazuh_control_path} execution..."
        
        local sudoers_line="${wazuh_user} ALL=(ALL) NOPASSWD: ${wazuh_control_path} *"
        
        # Create a temporary file first
        local tmp_sudoers
        tmp_sudoers=$(mktemp)
        echo "${sudoers_line}" > "${tmp_sudoers}"
        
        # Validate sudoers file before moving it (if visudo is available)
        if command -v visudo >/dev/null 2>&1; then
            if ! maybe_sudo visudo -cf "${tmp_sudoers}"; then
                error_message "Invalid sudoers configuration generated. Skipping sudoers setup."
                rm -f "${tmp_sudoers}"
                return 1
            fi
        fi

        maybe_sudo mv "${tmp_sudoers}" "${sudoers_file}"
        # Use GID 0 for the root group (root on Linux, wheel on macOS)
        maybe_sudo chown root:0 "${sudoers_file}"
        maybe_sudo chmod 0440 "${sudoers_file}"
        success_message "Sudoers configured: ${sudoers_file}"
    else
        info_message "User is root, skipping sudoers configuration."
    fi
    return 0
}

# Centralized permission and ownership setup for Wazuh-Agent-Status
setup_permissions_and_ownership() {
    local wazuh_user="${1}"
    local wazuh_group="${2}"
    local wazuh_control_path="${3}"
    local log_dir="/var/log/wazuh-agent-status"
    local log_file_path="${log_dir}/wazuh-agent-status.log"

    # 1. Adjust wazuh-control group to allow non-root execution
    if [[ -f "${wazuh_control_path}" ]]; then
        info_message "Adjusting ${wazuh_control_path} group to ${wazuh_group}..."
        maybe_sudo chgrp "${wazuh_group}" "${wazuh_control_path}"
        maybe_sudo chmod g+x "${wazuh_control_path}"
        success_message "wazuh-control permissions updated."
    else
        warn_message "${wazuh_control_path} not found, skipping."
    fi

    # 2. Adjust log file and directory permissions
    info_message "Ensuring log directory ${log_dir} has correct ownership..."
    if ! maybe_sudo [ -d "${log_dir}" ]; then
        maybe_sudo mkdir -p "${log_dir}"
    fi
    maybe_sudo chown "${wazuh_user}:${wazuh_group}" "${log_dir}"
    maybe_sudo chmod 775 "${log_dir}"

    info_message "Ensuring log file ${log_file_path} exists and has correct ownership..."
    if ! maybe_sudo [ -f "${log_file_path}" ]; then
        maybe_sudo touch "${log_file_path}"
    fi
    maybe_sudo chown "${wazuh_user}:${wazuh_group}" "${log_file_path}"
    maybe_sudo chmod 664 "${log_file_path}"
    success_message "Log file and directory permissions updated."

    # 3. Setup sudoers for self-healing (restarting the agent)
    setup_sudoers "${wazuh_user}" "${wazuh_control_path}"
}
