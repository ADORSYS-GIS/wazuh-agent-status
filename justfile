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

# Run tests per crate
test-proto:
    cargo test -p wazuh-status-proto-build

test-socket:
    cargo test -p wazuh-status-socket

test-core:
    cargo test -p wazuh-status-core

test-common:
    cargo test -p wazuh-status-common

test-daemon:
    cargo test -p wazuh-status-daemon

test-client:
    cargo test -p wazuh-status-client

test-all:
    just test-backend && just test-proto && just test-socket && just test-core && just test-common && just test-daemon && just test-client

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
