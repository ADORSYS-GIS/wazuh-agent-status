---
layout: default
title: Troubleshooting & FAQs
nav_order: 3
---

# Troubleshooting & FAQs

If you are experiencing issues with Wazuh Agent Status, check the following common solutions before reaching out to support.

## 1. Agent Shows as "Disconnected"

If the tray icon is red and the dashboard shows "Agent Disconnected", this means the Rust server is unable to communicate with the local Wazuh agent on your machine.

**Steps to Fix:**
1. **Verify Wazuh is Installed:** Make sure the actual Wazuh Agent is installed on your machine.
2. **Check the Service:** Ensure the Wazuh service is running.
   - **Windows:** Open `services.msc` and look for `Wazuh`.
   - **Linux:** Run `sudo systemctl status wazuh-agent`.
   - **macOS:** Run `sudo /Library/Ossec/bin/wazuh-control status`.
3. **Restart the Server:** If the Wazuh agent is running, try restarting the `wazuh-agent-status` server service.

## 2. AI Fixes: Rate Limit Exceeded

When attempting to generate a fix in the Compliance Dashboard, you may receive an error stating `AI Rate Limit Exceeded`.

**Reason:** 
This occurs when you have requested too many automated fixes within a short time frame, hitting the quota for the default API key configured in the desktop client.

**Steps to Fix:**
Currently, AI fixes are rate-limited per installation. If you require more fixes, you must wait for the cooldown period (usually 1 hour) before generating new fixes. In a future update, you will be able to supply your own API key in the configuration file.

## 3. Where are the log files?

If you need to debug the application or submit an issue on GitHub, you will likely need the log files. The Rust server and Tauri client keep their logs in standard application data directories.

- **Windows:** `%APPDATA%\wazuh-agent-status\logs\wazuh-agent-status.log`
- **Linux:** `~/.config/wazuh-agent-status/logs/wazuh-agent-status.log`
- **macOS:** `~/Library/Logs/wazuh-agent-status/wazuh-agent-status.log`

> **Tip**
> > You can also view real-time logs directly in the desktop app by clicking the **Log Stream** button in the tray menu.

---

## Still having trouble?

If these steps didn't solve your issue, please check our [GitHub Issues](https://github.com/ADORSYS-GIS/wazuh-agent-status/issues) page to see if someone else has reported the same problem, or open a new issue.
