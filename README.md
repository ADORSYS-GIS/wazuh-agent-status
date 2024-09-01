# Wazuh Agent Status

**Wazuh Agent Status** is an application designed to monitor the state of Wazuh agents. This tool provides real-time insights into the operational status of agents, ensuring that they are functioning correctly and efficiently.

## Features

- **Real-time Monitoring**: Continuously checks the status of Wazuh agents.
- **Build Pipeline**: Improved build pipeline for better performance.

## Supported Operating Systems
- **Ubuntu**

## Installation

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
   go get github.com/getlantern/systray
   go mod tidy
   go mod download

   sudo apt-get update
   sudo apt-get install -y libayatana-appindicator3-dev
   ```

## Building Binaries

You can build binaries for different systems by setting the `GOOS` and `GOARCH` environment variables before running the `go build` command. Here's how you can do it:

- For Linux (amd64):

  ```bash
  go build
  ```

Please note that you'll need to have a suitable C compiler installed in your system and be on an Ubuntu machine for this to work.

## Usage

Run the application with the following command:
```bash
./wazuh-tray 
```