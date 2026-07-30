# Wazuh Agent Status

[![Release Client](https://github.com/ADORSYS-GIS/wazuh-agent-status/actions/workflows/release.yaml/badge.svg)](https://github.com/ADORSYS-GIS/wazuh-agent-status/actions/workflows/release.yaml)

**Wazuh Agent Status** is a Rust-based application designed to monitor the state of Wazuh agents. It provides real-time insights into the operational status of agents through a system tray application with a modern web-based UI.

## Key Features

- **Real-time Status Monitoring:** Constantly monitors each Wazuh agent, updating statuses directly within your system tray.

- **Status and Connection Indicators:** Uses color-coded icons to show agent activity (Active/Inactive) and connection status (Connected/Disconnected).

- **Control Options:** Easily manage agents through the tray menu with options to restart or update the agent.

- **Compliance Dashboard:** View Security Configuration Assessment (SCA) results with detailed pass/fail information.

- **AI-Powered Remediation:** Automatically generate fix commands for failed compliance checks, with secure command execution and validation.

- **Real-time Log Streaming:** View Wazuh agent logs directly from the tray application.

- **Auto-Update Management:** Configure automatic updates or manually trigger stable/prerelease updates.

- **Cross-Platform Compatibility:** Native support for Linux, macOS, and Windows.

## Architecture

The system follows a **Client-Server** architecture:

- **Server (`wazuh-agent-status-rust-server`)**: A privileged background service that monitors the Wazuh agent's status, polls for updates every 5 seconds, and streams results to connected clients.
- **Client (`wazuh-agent-status-rust-client`)**: A Tauri-based desktop application with a system tray icon and a rich web UI for status monitoring, compliance checks, log viewing, and configuration.

Communication between client and server happens over localhost TCP (port 50505).

## Installation from source

### Prerequisites

- **Rust** (install via [rustup.rs](https://rustup.rs/))
- **Node.js** (for the Tauri client)
- **System dependencies** (Linux: `libsoup-3.0-dev`, `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`)

### Build the Server

```bash
cd wazuh-agent-status-rust-server
cargo build --release
```

### Build the Client (Tray App)

```bash
cd wazuh-agent-status-rust-client
npm install
npm run tauri build
```

## Quick Start

After building, start the server as a background service:

```bash
sudo ./target/release/wazuh-agent-status-rust-server
```

The client will start automatically from the system tray after installation.

## Automated Installation

Run the following command to install the app using the official script:

### Linux, macOS, and Windows

```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/install.sh | sh
```

## Documentation

- **[Architecture Overview](docs/architecture/architecture.md)**: High-level view of the system design.
- **[AI Compliance Fixes](docs/ai-compliance-fixes.md)**: Technical detail on AI fix generation and command execution.
- **[Rust Migration Proposal](docs/rust-migration-technical-proposal.md)**: Details of the completed Rust migration.
- **[Decision Log (ADRs)](docs/architecture/adr/)**: Architectural decision records.
- **[Roadmap](docs/roadmap.md)**: Future development plans.

## Project Structure

```
wazuh-agent-status/
├── wazuh-agent-status-rust-server/      # Rust TCP server (background service)
├── wazuh-agent-status-rust-client/       # Tauri desktop client (tray app)
├── scripts/                              # Install/uninstall/update scripts
│   ├── linux/
│   ├── macos/
│   ├── windows/
│   └── shared/
├── docs/                                 # Documentation
│   └── architecture/
│       ├── adr/                         # Architectural Decision Records
│       ├── architecture.md
│       ├── current-system-analysis.md
│       └── phase-1-rust-client-plan.md
├── .github/workflows/                    # CI/CD pipelines
├── CHANGELOG.md
├── cliff.toml                           # Changelog generator config
└── checksums.sha256                     # Release checksums
```
