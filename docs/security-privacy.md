---
layout: default
title: Security & Privacy
nav_order: 4
---

# Security & Privacy

Wazuh Agent Status is built with security and data privacy as core principles. Because this application interacts directly with your machine's local Wazuh agent, it requires privileged access, which we treat with the utmost care.

## 1. Gateway Data Access

All real-time monitoring and compliance data displayed in the application is fetched securely from the centralized Wazuh **Gateway**.

### How it works
The `wazuh-agent-status-rust-server` runs as a background service on your machine. It communicates with the remote Wazuh Gateway API to fetch information regarding your local agent's status.

To ensure this network communication cannot be spoofed or intercepted, the application connects to the Gateway over a secured connection. We rely on **HMAC authentication** to verify that the client is authorized to retrieve compliance data or issue commands for the agent.

> **⚠️ Important**
>
> Your raw compliance results and log streams are routed directly from the Wazuh Gateway to the Tauri desktop client, ensuring you see the exact same unified data as the central administration dashboard.
{: .important }

## 2. AI Remediation Privacy

In addition to syncing data with the Wazuh Gateway and checking GitHub for updates, the application communicates with an external AI service when you use the **AI-Powered Remediation** feature to generate a fix for a failed compliance check.

When you click "Generate Fix", we are extremely careful about the data transmitted to the AI provider.

### What is sent to the AI:
- **Check Title:** (e.g., "Ensure password length is greater than 14")
- **Check Description:** The generic policy description associated with the check.
- **Operating System:** (e.g., "Windows 11" or "Ubuntu 22.04") to ensure the generated shell command is compatible with your environment.

### What is NEVER sent to the AI:
- Your Wazuh agent ID, IP address, or hostnames.
- Real-time log data or event streams.
- The contents of your local files or specific configurations (we only send the generic title of the policy that failed, not the actual state of your machine).

## 3. Sandboxed Execution

When the AI returns a fix command, it is **never executed automatically**. 

1. The command is displayed to you in the UI for review.
2. It runs inside a restricted sub-shell managed by the Rust server.
3. If the command attempts to execute destructive binaries (e.g., `rm -rf`, `format`, or fetching external shell scripts), the Rust server will block the execution before it starts.
