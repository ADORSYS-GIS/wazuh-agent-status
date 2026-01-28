# Helper scripts for common build, run, compose, and test workflows.

set shell := ["bash", "-lc"]

#
# Desktop daemon helpers
#

# Build wazuh-status-daemon from the backend workspace
build-daemons:
    cargo build --release -p wazuh-status-daemon

# Run the desktop wazuh-status-daemon peer for local development
run-daemon:
    export WAZUH__AGENT_DIR="$PWD/.data" && cargo run -p wazuh-status-daemon -- --socket-path "/tmp/wazuh-status-daemon-dev.sock"

#
# Desktop icon helpers
#

# Generate icon assets for the Electron-based Status app
icons:
    pnpm --filter wazuh-status-client tauri icon app-icon.svg

# Run the full Rust backend test suite
test-backend:
    cargo test

# Run backend + desktop checks used in CI pipelines
ci-verify:
    just test-backend

run-status:
    pnpm --filter wazuh-status-client tauri dev

# Build the status app
build-status:
    pnpm --filter wazuh-status-client tauri build

# Show available just recipes
help:
    just --list
