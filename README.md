# Wazuh Agent Status

[![Release](https://github.com/ADORSYS-GIS/wazuh-agent-status/actions/workflows/release.yaml/badge.svg)](https://github.com/ADORSYS-GIS/wazuh-agent-status/actions/workflows/release.yaml)

**Wazuh Agent Status** is a Rust-based application designed to monitor the state of Wazuh agents. It provides real-time insights into the operational status of agents through a system tray application with a modern web-based UI.

## Key Features

- **Real-time Status Monitoring:** Constantly monitors each Wazuh agent, updating statuses directly within your system tray.
- **Status and Connection Indicators:** Uses color-coded icons to show agent activity (Active/Inactive) and connection status (Connected/Disconnected).
- **Control Options:** Easily manage the agent through the tray menu with options to check for updates or view the dashboard.
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

## Installation

The easiest way to install **Wazuh Agent Status** is via our automated installation scripts, which will download and configure the latest release for your platform.

### Linux

Open your terminal and run:

```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/user-main/scripts/linux/install.sh | sudo bash
```

### macOS

Open your terminal and run:

```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/user-main/scripts/macos/install.sh | sudo bash
```

### Windows

Open **PowerShell as Administrator** and run:

```powershell
Invoke-RestMethod -Uri "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/user-main/scripts/windows/install.ps1" | Invoke-Expression
```

*(Note: The Windows installation automatically configures the server as a Windows Service and sets the client to start automatically on login.)*

## Building and Running from Source

Follow these step-by-step instructions to build the application yourself or contribute to the project.

### Step 1: Install Prerequisites

Ensure you have the following installed on your system:
- **Rust**: Install via [rustup.rs](https://rustup.rs/)
- **Node.js**: Required for building the Tauri client UI (version 18+ recommended)
- **Platform Specific Dependencies**:
  - **Linux**: `libsoup-3.0-dev`, `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev` (Debian/Ubuntu)
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Windows**: Build Tools for Visual Studio 2022 (with C++ desktop development)

### Step 2: Clone the Repository

```bash
git clone https://github.com/ADORSYS-GIS/wazuh-agent-status.git
cd wazuh-agent-status
```

### Step 3: Configure the Port (Optional)

By default, the client and server communicate over TCP port `50505`. If you need to change this port, you must configure both the client and server.

1. **Update the Client Configuration:**
   Edit the file `wazuh-agent-status-rust-client/src-tauri/app_config.json` and change `"server_addr"` to your desired port:
   ```json
   {
     "server_addr": "127.0.0.1:50506"
   }
   ```

2. **Run the Server with a Custom Port (Step 4 details this command):**
   You will pass the `WAZUH_STATUS_ADDR` environment variable when starting the server.

### Step 4: Build and Run the Server

The server acts as the background service monitoring the Wazuh agent.

**Build:**
```bash
cd wazuh-agent-status-rust-server
cargo build --release
```

**Run (with default port 50505):**
- **Linux/macOS:** `sudo ./target/release/wazuh-agent-status-rust-server`
- **Windows:** `.\target\release\wazuh-agent-status-rust-server.exe` (Run as Administrator)

**Run (with custom port 50506):**
- **Linux/macOS:** `sudo WAZUH_STATUS_ADDR="127.0.0.1:50506" ./target/release/wazuh-agent-status-rust-server`
- **Windows (PowerShell):** `$env:WAZUH_STATUS_ADDR="127.0.0.1:50506"; .\target\release\wazuh-agent-status-rust-server.exe` (Run as Administrator)

### Step 5: Build and Run the Client (Tray App)

Open a new terminal window to build and run the client.

**Build:**
```bash
cd wazuh-agent-status-rust-client
npm install
npm run tauri build
```
*(On Windows, ensure you run these commands in PowerShell or Git Bash).*

**Run in Development Mode:**
To quickly test the client without running the compiled executable, use:
```bash
npm run tauri dev
```

**Run Compiled Executable:**
Alternatively, run the generated executable located in the `src-tauri/target/release/` directory.

## Documentation

- **[Architecture Overview](docs/architecture/architecture.md)**: High-level view of the system design.
- **[AI Compliance Fixes](docs/ai-compliance-fixes.md)**: Technical detail on AI fix generation and command execution.
- **[Rust Migration Proposal](docs/rust-migration-technical-proposal.md)**: Details of the completed Rust migration.
- **[Decision Log (ADRs)](docs/architecture/adr/)**: Architectural decision records.
- **[Roadmap](docs/roadmap.md)**: Future development plans.

## Project Structure

```text
wazuh-agent-status/
├── wazuh-agent-status-rust-server/      # Rust TCP server (background service)
├── wazuh-agent-status-rust-client/      # Tauri desktop client (tray app)
├── scripts/                             # Install/uninstall/update scripts
│   ├── linux/
│   ├── macos/
│   ├── windows/
│   └── shared/
├── docs/                                # Documentation
│   └── architecture/
│       ├── adr/                         # Architectural Decision Records
│       ├── architecture.md
│       ├── current-system-analysis.md
│       └── phase-1-rust-client-plan.md
├── .github/workflows/                   # CI/CD pipelines
├── CHANGELOG.md                         # Auto-generated changelog
├── cliff.toml                           # Changelog generator config
└── checksums.sha256                     # Release checksums
```
