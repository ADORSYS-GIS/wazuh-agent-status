---
layout: default
title: Security & Privacy
nav_order: 4
---

# Security & Privacy

Wazuh Agent Status is built with security and data privacy as core principles. Because this application interacts directly with your machine's local Wazuh agent, it requires privileged access, which we treat with the utmost care.

## 1. Local Data Access

All real-time monitoring and compliance data displayed in the application is fetched **locally**. 

### How it works
The `wazuh-agent-status-rust-server` runs as a privileged background service on your machine. It communicates directly with the local Wazuh agent via internal APIs. 

To ensure this local communication cannot be spoofed or intercepted, the client application connects to the background service over a secured local TCP connection. We rely on **HMAC authentication** to verify that only authorized local clients can retrieve compliance data or issue commands to the agent.

> **Important**
> > Your raw compliance results and local log streams never leave your machine. They are routed directly from the Wazuh agent to the Tauri desktop client.

## 2. AI Remediation Privacy

The only time Wazuh Agent Status communicates with an external service (aside from checking GitHub for application updates) is when you use the **AI-Powered Remediation** feature to generate a fix for a failed compliance check.

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
