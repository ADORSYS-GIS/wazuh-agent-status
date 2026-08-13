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
    return 1
}

success_message() {
    log "${GREEN}${BOLD}[SUCCESS]${NORMAL}" "$*"
    return 0
}

# ── Deep debug logging ────────────────────────────────────────────────────────
#
# Debug mode is enabled when WAZUH_AGENT_STATUS_DEBUG is a truthy value OR when
# a marker file exists. The marker file is essential for the UI-triggered update
# path: that chain runs the script through several 'sudo env VAR=... bash ...'
# boundaries (adorsys-update.sh -> setup-agent.sh -> install.sh) that drop
# unlisted environment variables. A file survives those boundaries, so:
#
#   Terminal test:  export WAZUH_AGENT_STATUS_DEBUG=1
#   UI/daemon test: sudo touch /tmp/wazuh-agent-status-debug
#
# The marker lives in /tmp so it is automatically cleared on reboot and can
# never accidentally stay enabled.
is_debug() {
    case "${WAZUH_AGENT_STATUS_DEBUG:-}" in
        1|true|TRUE|yes|YES|on|ON|debug|DEBUG) return 0 ;;
    esac
    if [[ -f "${WAZUH_AGENT_STATUS_DEBUG_FILE:-/tmp/wazuh-agent-status-debug}" ]]; then
        return 0
    fi
    return 1
}

debug_message() {
    if is_debug; then
        log "${BOLD}[DEBUG]${NORMAL}" "$*"
    fi
    return 0
}

# Variant for functions whose stdout is consumed by the caller (e.g.
# get_real_user, maybe_sudo): the debug line goes to stderr instead so it can
# never corrupt a captured value.
debug_message_err() {
    if is_debug; then
        log "${BOLD}[DEBUG]${NORMAL}" "$*" >&2
    fi
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

# Ensure the script is running on the expected operating system
ensure_os() {
    local expected_os="$1"
    local actual_os
    actual_os=$(uname -s)

    if [[ "$actual_os" != "$expected_os" ]]; then
        local os_name
        case "$expected_os" in
            Darwin) os_name="macOS" ;;
            Linux)  os_name="Linux" ;;
            *)      os_name="$expected_os" ;;
        esac
        error_exit "This script is intended for ${os_name} systems. Detected OS: ${actual_os}. Please use the appropriate script for your operating system."
    fi
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
            error_exit "Unsupported architecture: $arch"
            ;;
    esac
    return 0
}

# Check if sudo is available or if the script is run as root.
# Propagates the wrapped command's exit code (previously swallowed as 0 when
# running via sudo, which silently hid real failures).
maybe_sudo() {
    if [[ "$(id -u)" -ne 0 ]]; then
        debug_message_err "maybe_sudo: running as $(id -un) (uid $(id -u)); wrapping via sudo: $*"
        if command_exists sudo; then
            sudo "$@"
            return $?
        else
            error_exit "This script requires root privileges. Please run with sudo or as root."
        fi
    else
        debug_message_err "maybe_sudo: running as root; executing directly: $*"
        "$@"
        return $?
    fi
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
        error_message "Failed to download ${name} from ${url}"
        return 1
    fi

    # Handle external checksum file download if a URL is provided
    if [[ -n "${checksum_url}" ]]; then
        local temp_checksum_file
        temp_checksum_file=$(mktemp)
        if ! download_file "${checksum_url}" "${temp_checksum_file}" "checksum file"; then
            error_message "Failed to download external checksum file from ${checksum_url}"
            return 1
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
                error_message "Invalid checksum entry in ${checksum_file}"
                return 1
            fi

            if ! verify_checksum "${dest}" "${expected}"; then
                error_message "${name} checksum verification FAILED"
                return 1
            fi
            info_message "${name} checksum verification passed."
        else
            error_message "No checksum found for ${name} in ${checksum_file} with pattern '${pattern}'"
            error_message "First 10 lines of the checksum file for debugging:"
            head -n 10 "${checksum_file}"
            error_message "Checksum lookup failed for ${name}"
            return 1
        fi

        # Clean up temporary checksum files
        if [[ -n "${checksum_url}" ]] && [[ -f "${checksum_file}" ]]; then
            rm -f "${checksum_file}"
        fi
    else
        error_message "Checksum file not found at ${checksum_file}; cannot verify ${name}"
        return 1
    fi

    success_message "${name} downloaded and verified successfully."
    return 0
}

# Validate that a URL uses HTTPS (to satisfy SonarCloud HTTPS enforcement)
enforce_https_url() {
    local url="$1"
    local name="${2:-URL}"
    case "$url" in
        https://*) : ;;
        *) echo "[ERROR] $name must use HTTPS. Got: $url" >&2; exit 1 ;;
    esac
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
    return 0
}

# Detect the real user who invoked the script (even via sudo).
# NOTE: this function's stdout is its return value, so all debug output goes
# to stderr via debug_message_err and can never leak into the result.
get_real_user() {
    debug_message_err "get_real_user: SUDO_USER='${SUDO_USER:-}' LOGNAME='${LOGNAME:-}' USER='${USER:-}' os='$(uname -s)'"

    # If SUDO_USER is set to a real (non-root) user, trust it. When the update
    # chain runs 'sudo env VAR=... bash ...' as root (server -> adorsys-update.sh
    # -> setup-agent.sh -> install.sh), sudo reports SUDO_USER=root, which tells
    # us nothing about the actual GUI session user. Trusting it would point the
    # macOS launchctl 'gui' load at the wrong session, so fall through to the
    # console-user detection below in that case.
    if [[ -n "${SUDO_USER:-}" ]] && [[ "$SUDO_USER" != "root" ]]; then
        debug_message_err "get_real_user: using SUDO_USER='$SUDO_USER'"
        echo "$SUDO_USER"
        return
    fi

    # Check LOGNAME or USER if they are not root
    if [[ -n "${LOGNAME:-}" ]] && [[ "$LOGNAME" != "root" ]]; then
        debug_message_err "get_real_user: using LOGNAME='$LOGNAME'"
        echo "$LOGNAME"
        return
    fi
    if [[ -n "${USER:-}" ]] && [[ "$USER" != "root" ]]; then
        debug_message_err "get_real_user: using USER='$USER'"
        echo "$USER"
        return
    fi

    # Fallback for Linux using process tree
    if [[ "$(uname -s)" == "Linux" ]]; then
        local pid=$$
        while [[ "$pid" -ne 1 ]] && [[ -n "$pid" ]]; do
            pid=$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ')
            [[ -z "$pid" ]] && break
            local user
            user=$(ps -o user= -p "$pid" 2>/dev/null | tr -d ' ')
            if [[ -n "$user" ]] && [[ "$user" != "root" ]]; then
                debug_message_err "get_real_user: Linux process tree found non-root ancestor '$user' (ppid $pid)"
                echo "$user"
                return
            fi
        done
        debug_message_err "get_real_user: Linux process tree yielded no non-root ancestor"
    fi

    # Fallback for macOS using stat on the tty
    if [[ "$(uname -s)" == "Darwin" ]]; then
        local tty_user
        tty_user=$(stat -f "%Su" /dev/console 2>/dev/null)
        debug_message_err "get_real_user: console user from stat /dev/console='${tty_user:-<none or unreadable>}'"
        if [[ -n "$tty_user" ]] && [[ "$tty_user" != "root" ]]; then
            debug_message_err "get_real_user: using console user '$tty_user'"
            echo "$tty_user"
            return
        fi
    fi

    # Last resort: current user (might be root)
    local fallback
    fallback=$(id -un 2>/dev/null || true)
    [[ -n "$fallback" ]] || fallback="root"
    debug_message_err "get_real_user: fallback to current user '$fallback'"
    echo "$fallback"
    return 0
}

# ── Timeout & launchd management ─────────────────────────────────────────────

# Run a command with a hard timeout (pure bash; macOS ships no GNU 'timeout').
# Returns the command's exit code, or 124 if the timeout was hit.
# Usage: run_with_timeout <seconds> <command...>
run_with_timeout() {
    local timeout_secs="${1:?Usage: run_with_timeout <seconds> <command...>}"
    shift
    local pid
    local watcher
    local exit_code=0

    # Run the command in its own process group so the timeout watcher can kill
    # the whole tree (wrapper + sudo + launchctl), not just the wrapper.
    set -m
    "$@" &
    pid=$!
    set +m

    (
        sleep "$timeout_secs"
        kill -TERM -- "-$pid" 2>/dev/null || true
        sleep 1
        kill -KILL -- "-$pid" 2>/dev/null || true
    ) &
    watcher=$!

    wait "$pid" 2>/dev/null || exit_code=$?
    kill "$watcher" 2>/dev/null || true
    wait "$watcher" 2>/dev/null || true

    # 143 (SIGTERM) / 137 (SIGKILL) mean our watcher terminated it → timeout.
    if [[ "$exit_code" -eq 143 ]] || [[ "$exit_code" -eq 137 ]]; then
        return 124
    fi
    return "$exit_code"
}

# Run a command via run_with_timeout, replaying its captured output at debug
# level when debug mode is on. In normal mode the output is discarded (same as
# the historical '2>/dev/null' behaviour) so production transcripts stay clean;
# with WAZUH_AGENT_STATUS_DEBUG set, every launchctl call shows the exact
# command, its captured stdout/stderr, and its exit code / timeout verdict.
# Returns the command's exit code, or 124 if it was killed by the timeout.
# Usage: trace_run <seconds> <description> <command...>
trace_run() {
    local timeout_secs="${1:?Usage: trace_run <seconds> <description> <command...>}"
    local description="${2:?Usage: trace_run <seconds> <description> <command...>}"
    shift 2
    local rc=0
    local tmp_out=""

    if is_debug; then
        debug_message "trace_run [${description}]: executing (timeout ${timeout_secs}s): $*"
        tmp_out=$(mktemp "${TMPDIR:-/tmp}/wazuh-trace.XXXXXX") || tmp_out="/tmp/wazuh-trace.$$"
        run_with_timeout "$timeout_secs" "$@" >"$tmp_out" 2>&1 || rc=$?
        if [[ -s "$tmp_out" ]]; then
            while IFS= read -r line; do
                debug_message "[${description}] ${line}"
            done < "$tmp_out" || true
        fi
        rm -f "$tmp_out"
        if [[ "$rc" -eq 124 ]]; then
            debug_message "[${description}] TIMED OUT after ${timeout_secs}s and was terminated (rc=124)"
        else
            debug_message "[${description}] finished with rc=${rc}"
        fi
    else
        run_with_timeout "$timeout_secs" "$@" >/dev/null 2>&1 || rc=$?
    fi
    return "$rc"
}

# Idempotently load (or kickstart) a launchd service into the system or the
# logged-in user's GUI domain. Centralises all launchctl interaction so every
# caller gets the same timeout protection and clear error reporting. It never
# blocks indefinitely: every launchctl call is wrapped in run_with_timeout and
# all failures surface as explicit warnings instead of being silenced.
# Usage: manage_launch_service <system|gui> <label> <plist>
# Returns 0 on success, 1 on failure (after printing a warning).
manage_launch_service() {
    local domain="${1:?Usage: manage_launch_service <system|gui> <label> <plist>}"
    local label="${2:?Usage: manage_launch_service <system|gui> <label> <plist>}"
    local plist="${3:?Usage: manage_launch_service <system|gui> <label> <plist>}"
    local timeout_secs=15
    local target=""
    local real_user=""
    local uid=""
    local loaded=false
    local rc=0

    debug_message "manage_launch_service: domain='$domain' label='$label' plist='$plist' timeout=${timeout_secs}s"
    debug_message "manage_launch_service: process context: euid=$(id -u) user=$(id -un 2>/dev/null || true) HOME='${HOME:-<unset>}' SUDO_USER='${SUDO_USER:-<unset>}'"

    if [[ "$domain" == "gui" ]]; then
        real_user=$(get_real_user)
        debug_message "manage_launch_service: resolved GUI user via get_real_user -> '${real_user}'"
        if [[ -z "$real_user" ]] || [[ "$real_user" == "root" ]] || [[ "$real_user" == "loginwindow" ]]; then
            debug_message "manage_launch_service: rejecting GUI user '${real_user:-<none>}' (no active GUI session)"
            warn_message "Cannot load $label: no active GUI user found (console user: '${real_user:-none}'). The tray app will load after login."
            return 1
        fi
        uid=$(id -u "$real_user" 2>/dev/null || true)
        debug_message "manage_launch_service: id -u '$real_user' -> '${uid:-<unresolved>}'"
        if [[ -z "$uid" ]]; then
            debug_message "manage_launch_service: could not resolve UID for '$real_user'"
            warn_message "Cannot load $label: could not resolve UID for user '$real_user'."
            return 1
        fi
        target="gui/$uid/$label"
    else
        target="system/$label"
    fi
    debug_message "manage_launch_service: resolved launchd target: '$target'"

    # Already loaded? Two probes for the gui domain to avoid false 'not loaded'
    # verdicts when running from a daemon context.
    if trace_run "$timeout_secs" "print-probe-as-root $target" maybe_sudo launchctl print "$target"; then
        loaded=true
        debug_message "manage_launch_service: probe as root reports '$target' IS loaded"
    elif [[ "$domain" == "gui" ]] && trace_run "$timeout_secs" "print-probe-as-user $target" sudo -u "$real_user" launchctl print "$target"; then
        loaded=true
        debug_message "manage_launch_service: probe as user '$real_user' reports '$target' IS loaded"
    else
        debug_message "manage_launch_service: probes report '$target' NOT loaded"
    fi

    if [[ "$loaded" == "true" ]]; then
        info_message "Service $label is already loaded ($target), kickstarting..."
        if ! trace_run "$timeout_secs" "kickstart $target" maybe_sudo launchctl kickstart -k "$target"; then
            warn_message "Kickstarting $label failed. Run 'launchctl print $target' for details."
            return 1
        fi
        debug_message "manage_launch_service: kickstart '$target' succeeded"
        return 0
    fi

    info_message "Loading $label into ${domain} session..."
    # Note: '|| rc=$?' (not a bare 'rc=$?') so a failure can never trip 'set -e'
    # and abort the whole script — failures must degrade to a warning instead.
    if [[ "$domain" == "gui" ]]; then
        debug_message "manage_launch_service: bootstrap attempt 1/2: launchctl asuser '$uid' launchctl bootstrap 'gui/$uid' '$plist'"
        trace_run "$timeout_secs" "bootstrap-asuser $target" maybe_sudo launchctl asuser "$uid" launchctl bootstrap "gui/$uid" "$plist" \
            || trace_run "$timeout_secs" "bootstrap-as-user $target" sudo -u "$real_user" launchctl bootstrap "gui/$uid" "$plist" \
            || rc=$?
    else
        debug_message "manage_launch_service: bootstrapping '$plist' into the system domain"
        trace_run "$timeout_secs" "bootstrap-system $target" maybe_sudo launchctl bootstrap "system" "$plist" \
            || rc=$?
    fi

    if [[ "$rc" -eq 0 ]]; then
        debug_message "manage_launch_service: loaded '$target' successfully"
        return 0
    fi

    # The most common reason a GUI bootstrap fails during an update is that the
    # service is ALREADY loaded (the tray app is running) but the probes above
    # missed it — for example when running as root from the system/daemon
    # context. 'launchctl bootstrap' then fails with "service already loaded".
    # Boot the service out (best-effort) and retry the bootstrap once so the
    # client is restarted from the freshly written plist with the new binary.
    if [[ "$rc" -ne 124 ]] && [[ "$domain" == "gui" ]]; then
        debug_message "manage_launch_service: bootstrap failed (rc=$rc); booting out '$target' and retrying once"
        trace_run "$timeout_secs" "bootout-retry $target" maybe_sudo launchctl bootout "gui/$uid/$label" || true
        local retry_rc=0
        trace_run "$timeout_secs" "bootstrap-retry $target" maybe_sudo launchctl bootstrap "gui/$uid" "$plist" || retry_rc=$?
        if [[ "$retry_rc" -eq 0 ]]; then
            info_message "Service $label loaded successfully after reloading (target: $target)."
            return 0
        fi
        rc="$retry_rc"
        debug_message "manage_launch_service: bootstrap retry after bootout also failed (rc=$retry_rc)"
    fi

    if [[ "$rc" -eq 124 ]]; then
        warn_message "Loading $label timed out after ${timeout_secs}s. Target: $target. You may need to log out and log back in to see the tray app."
    else
        warn_message "Loading $label failed (exit $rc). Target: $target. You may need to log out and log back in to see the tray app."
    fi
    return 1
}

# Grant passwordless sudo for wazuh-control and update scripts to the wazuh user
setup_sudoers() {
    local wazuh_user="${1}"
    local wazuh_control_path="${2}"
    local sudoers_file="/etc/sudoers.d/wazuh-agent-status"

    # Determine the correct update script path based on the OS
    local adorsys_script_path
    if [[ "$(uname -s)" == "Darwin" ]]; then
        adorsys_script_path="/Library/Ossec/active-response/bin/adorsys-update.sh"
    else
        adorsys_script_path="/var/ossec/active-response/bin/adorsys-update.sh"
    fi

    # Only configure sudoers if the user is not root
    if [[ "${wazuh_user}" != "root" ]]; then
        info_message "Configuring sudoers for ${wazuh_user} to allow passwordless execution..."
        
        # Generate all sudoers lines
        {
            echo "# Wazuh Agent Status sudoers rules"
            echo "# Allow restarting the agent service"
            echo "${wazuh_user} ALL=(ALL) NOPASSWD: ${wazuh_control_path} *"
            echo ""
            echo "# Allow running the adorsys update script (stable updates)"
            echo "${wazuh_user} ALL=(ALL) NOPASSWD: ${adorsys_script_path}"
            echo ""
            echo "# Allow running downloaded setup scripts (prerelease updates)"
            echo "${wazuh_user} ALL=(ALL) NOPASSWD: /tmp/setup-agent-*.sh"
        } > "${sudoers_file}.tmp"
        
        # Validate sudoers file before moving it (if visudo is available)
        if command -v visudo >/dev/null 2>&1 && ! maybe_sudo visudo -cf "${sudoers_file}.tmp"; then
            error_message "Invalid sudoers configuration generated. Skipping sudoers setup."
            rm -f "${sudoers_file}.tmp"
            return 1
        fi

        maybe_sudo mv "${sudoers_file}.tmp" "${sudoers_file}"
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
    if ! maybe_sudo test -d "${log_dir}"; then
        maybe_sudo mkdir -p "${log_dir}"
    fi
    maybe_sudo chown "${wazuh_user}:${wazuh_group}" "${log_dir}"
    maybe_sudo chmod 775 "${log_dir}"

    info_message "Ensuring log file ${log_file_path} exists and has correct ownership..."
    if ! maybe_sudo test -f "${log_file_path}"; then
        maybe_sudo touch "${log_file_path}"
    fi
    maybe_sudo chown "${wazuh_user}:${wazuh_group}" "${log_file_path}"
    maybe_sudo chmod 664 "${log_file_path}"
    success_message "Log file and directory permissions updated."

    # 3. Setup sudoers for self-healing (restarting the agent)
    setup_sudoers "${wazuh_user}" "${wazuh_control_path}"
    return 0
}
