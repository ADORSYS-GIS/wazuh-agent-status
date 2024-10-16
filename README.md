# Wazuh Agent Status
[![Release Client](https://github.com/ADORSYS-GIS/wazuh-agent-status/actions/workflows/release.yaml/badge.svg)](https://github.com/ADORSYS-GIS/wazuh-agent-status/actions/workflows/release.yaml)

**Wazuh Agent Status** is an application designed to monitor the state of Wazuh agents. This tool provides real-time insights into the operational status of agents, ensuring they are functioning correctly and efficiently.

## Features

- **Real-time Monitoring**: Continuously checks the status of Wazuh agents.
- **Multi-platform Support**: Works on **Linux** and **macOS**.

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

## Usage

Run the application with the following command:
```bash
./dist/wazuh-agent-status
```

## Alternative Installation

To install the agent companion, run the script that will download and install it for you:
```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/main/scripts/install.sh | bash
```

## Usage

After installation, you can run the application with:
```bash
wazuh-agent-status
```