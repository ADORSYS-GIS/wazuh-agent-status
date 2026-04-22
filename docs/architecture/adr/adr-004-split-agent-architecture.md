# ADR-004: Use Split-Agent Architecture

## Status
Accepted

## Context
The application needs to perform privileged operations (reading Wazuh config, restarting services) while providing a user-friendly tray interface running in user space.

## Decision
Implement a "Split-Agent" architecture consisting of:
1.  **Background Server**: A privileged service handling system interactions and Wazuh status monitoring.
2.  **Tray Client**: A lightweight user-space application providing the UI.

## Consequences
- **Security**: Privileged operations are isolated in the server.
- **Reliability**: UI crashes do not affect the monitoring service.
- **Resource Usage**: Both components can be optimized independently.
