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


## Installation from code

To install the application, follow these steps:

1. Clone the repository:
   ```bash
   git clone https://github.com/ADORSYS-GIS/wazuh-agent-status.git
   ```
2. Navigate to the project directory:
   ```bash
   cd wazuh-agent-status
   ```
3. Install dependencies:
   ```bash
   go mod tidy
   go mod download
   
   ### Additional steps for Ubuntu
   sudo apt-get update
   sudo apt-get install -y libayatana-appindicator3-dev
   ```

## Building Binaries

You can build binaries for different systems using this command:

- Cross Platform build For Linux, macOS and Windows:
  ```bash
    GOOS=linux GOARCH=amd64 go build -o dist/wazuh-agent-status-linux
    GOOS=darwin GOARCH=amd64 go build -o dist/wazuh-agent-status-macos
    GOOS=windows GOARCH=amd64 go build -o dist/wazuh-agent-status-windows
  ```

> **Note**: Ensure you have a suitable C compiler installed on your system for this to work.

## Quick Start
After building or installing the binary, you can start the application as follows:

Run the application with:
```bash
./dist/wazuh-agent-status
```

### Alternative Installation

To install the agent companion, run the script that will download and install it for you:
```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/install.sh | bash
```

After installation, launch the application with:
```bash
wazuh-agent-status
```
