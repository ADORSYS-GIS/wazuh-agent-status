# Wazuh Agent Status

[![Release Client](https://github.com/ADORSYS-GIS/wazuh-agent-status/actions/workflows/release.yaml/badge.svg)](https://github.com/ADORSYS-GIS/wazuh-agent-status/actions/workflows/release.yaml)

**Wazuh Agent Status** is an application designed to monitor the state of Wazuh agents. This tool provides real-time insights into the operational status of agents, ensuring they are functioning correctly and efficiently.

## Key Features

- **Real-time Status Monitoring:** Constantly monitors each Wazuh agent, updating statuses directly within your system tray.

- **Status and Connection Indicators:** Uses color-coded icons to show agent activity (Active/Inactive) and connection status (Connected/Disconnected).

- **Control Options:** Easily manage agents through the tray menu with options to pause, restart, or quit the agent.

- **Cross-Platform Compatibility:** Compatible with Linux, macOS, and Windows.

- **Embedded Icons:** Custom, embedded icons ensure immediate visual recognition for easy status assessment.

## Core Functionalities

- **Instant Agent Status Updates:** The system tray displays live updates on each agent’s operational state, ensuring administrators can act promptly if issues arise.

- **Connection Health Validation:** Regular checks confirm each agent’s connection integrity, with clear indications for connection loss or re-establishment.

## Installation from source

To build the application from source, follow these steps:

1. Clone the repository:
   ```bash
   git clone https://github.com/ADORSYS-GIS/wazuh-agent-status.git
   ```
2. Navigate to the project directory:
   ```bash
   cd wazuh-agent-status
   ```
3. Install Rust (if not already installed) via [rustup.rs](https://rustup.rs/).
4. Install Node.js for the Tauri client.

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

After building, you can start the server as follows:

```bash
sudo ./target/release/wazuh-agent-status-rust-server
```

## Automated Installation

Run the following command to install the app using the official script:

- ### Linux, macOS, and Windows
  ```bash
  curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/install.sh | sh
  ```

## 📖 Documentation Guide

To understand the project vertically—from the business vision down to the technical code decisions—we recommend reading the documentation in this order:

1.  **[Improvement Roadmap](docs/roadmap.md)**: Start here to understand the 6-phase strategic vision.
2.  **[Architecture Overview](docs/architecture/architecture.md)**: A high-level view of the "Brains & Face" design.
3.  **[Rust Migration Technical Proposal](docs/rust-migration-technical-proposal.md)**: Detailed strategy for the Rust transition.
4.  **[Self-Healing Design](docs/architecture/adr/self-healing.md)**: Details on the reactive health orchestration.
5.  **[Decision Log (ADRs)](docs/architecture/adr/)**: A record of architectural decisions.

---

After installation, the server runs as a background service, and the client starts automatically on login.
