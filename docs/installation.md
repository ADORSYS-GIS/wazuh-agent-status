---
layout: default
title: Installation & Setup
nav_order: 2
---

# Installation & Setup

This guide provides step-by-step instructions on how to install the complete **Wazuh Agent Status** stack across different platforms. The automated installation scripts will deploy the Wazuh Agent, the privileged Rust backend service, and the desktop client in one go.

## Prerequisites

Because the automated installation scripts handle the deployment of the entire stack, there are no prior software dependencies required. The scripts will automatically download and install the **Wazuh Agent** for you if it is not already present on your machine.

## Platform Installation

The easiest way to install the application is via our automated installation scripts, which will download, verify, and configure the latest release for your platform.

### Linux (Debian/Ubuntu)

Open your terminal and run the following command:

```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/user-main/scripts/linux/install.sh | sudo bash
```

> **ℹ️ Note**
>
> This script installs the `.deb` package and sets up the background service using `systemd`.
{: .note }

### macOS

Open your terminal and run the following command:

```bash
curl -sL https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/user-main/scripts/macos/install.sh | sudo bash
```

> **ℹ️ Note**
>
> The macOS script downloads the `.dmg`, installs the application to `/Applications`, and configures `launchd` for the background service.
{: .note }

### Windows

Open **PowerShell as Administrator** and run the following command:

```powershell
Invoke-RestMethod -Uri "https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent-status/user-main/scripts/windows/install.ps1" | Invoke-Expression
```

> **⚠️ Important**
>
> You must run this command in an **elevated** PowerShell session (Run as Administrator) to allow the installer to register the Windows Service.
{: .important }

---

## Configuration: Self-Healing

Wazuh Agent Status includes a **Self-Healing** feature. If enabled, the background server will automatically attempt to restart the local Wazuh agent if it detects that the service has unexpectedly stopped.

### How to Enable

To enable self-healing, you need to set the `WAZUH_STATUS_SELF_HEALING` environment variable to `true` for the background service.

**On Linux (`systemd`):**
1. Edit the service file: `sudo systemctl edit wazuh-agent-status`
2. Add the following block:
   ```ini
   [Service]
   Environment="WAZUH_STATUS_SELF_HEALING=true"
   ```
3. Restart the service: `sudo systemctl restart wazuh-agent-status`

**On Windows (`Service Manager`):**
1. Open the System Properties > Environment Variables.
2. Under "System variables", click "New".
3. Set the Variable name to `WAZUH_STATUS_SELF_HEALING` and value to `true`.
4. Restart the `wazuh-agent-status` service via the Windows Services snap-in (`services.msc`).

> **💡 Tip**
>
> You can verify self-healing is active by manually stopping the Wazuh agent service and watching the tray icon quickly turn back to green.
{: .note }
