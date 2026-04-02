# 🏗️ Architecture Document — Wazuh Agent Status

## 1. Introduction

This document describes the architecture of the **Wazuh Agent Status** system, including the current implementation and the target hardened architecture.

The system provides visibility and control over the Wazuh agent via a local tray application and a privileged background service.

---

## 2. The Concept: "Brains & Face"

To understand the system simply, we can think of it as two parts:

- **The Brains (Server)**: A background service that handles the heavy lifting—talking to the OS, managing service states, and interacting with the Wazuh Agent's files. While it performs privileged OS operations, it can be configured to run with restricted permissions by being added to the appropriate group (e.g., `ossec`).
- **The Face (Client)**: A lightweight tray application that runs in the "User Space." It’s designed to be simple and accessible, showing the user only what they need to see (a green/red dot) and giving them easy buttons to click.

By splitting the app this way, we ensure that the "Face" remains responsive and safe, while the "Brains" handles the complex security operations securely.

---

## 3. Requirements

### 3.1 Functional Requirements

- Display Wazuh agent status
- Show connection state
- Display client version
- Allow restart/update actions
- Provide auto-update capability
- Show "Update to Prerelease" menu item if available

### 3.2 Non-Functional Requirements

- Security (authentication, encryption)
- Performance (low CPU and memory usage)
- Cross-platform compatibility (Windows, Linux, macOS)
- Reliability and fault tolerance
- Maintainability

---

## 4. Constraints

- Must run locally on user machines
- Must interact with Wazuh agent
- Must support multiple operating systems
- Must operate with least privilege principles

---

## 5. System Overview

The system follows a **Client-Server (Split-Agent)** architecture:

- **Client (Tray App)**: UI layer running in user space
- **Server (Background Service)**: privileged component interacting with the OS and Wazuh agent

---

## 6. Current Architecture

The current Go implementation uses a **Push/Subscribe** model over TCP (port 50505). While the server polls the OS every 5 seconds to detect changes, it pushes those changes instantly to all connected clients, ensuring the UI is reactive.

For a detailed technical breakdown of the performance and security limitations of the current system, see:
👉 **[Current System Analysis](current-system-analysis.md)**

---

## 7. Target Architecture (Hardened)

Our long-term goal is a Zero-Trust, event-driven architecture based on Rust and gRPC. For a complete technical breakdown of this transition, see:
👉 **[Rust Migration Proposal](../rust-migration-technical-proposal.md)**

---

## 8. Technology Stack

| Layer         | Technology                   |
| ------------- | ---------------------------- |
| Client        | Rust (tray-icon, tokio)      |
| Server        | Go (current) → Rust (target) |
| Communication | TCP → gRPC                   |
| Security      | TLS / mTLS                   |
| Serialization | Plaintext → Protobuf         |

---

## 9. Data Flow

1. Client sends request to server
2. Server validates request
3. Server queries Wazuh agent / OS
4. Server returns response
5. Client updates UI

---

## 10. Security Considerations

- Encrypted communication (TLS)
- Mutual authentication (mTLS)
- Localhost binding
- Audit logging
- Least privilege enforcement

---

## 11. Risks & Trade-offs

| Risk                       | Impact                | Mitigation                   |
| -------------------------- | --------------------- | ---------------------------- |
| Rust learning curve        | Slower development    | Training + gradual migration |
| Cross-platform differences | Inconsistent behavior | OS-specific testing          |
| Tray support limitations   | UI issues             | Use stable libraries         |

---

## 12. Future Improvements

- Event-driven architecture
- Real-time updates (streaming)
- Extended system monitoring
- Plugin-based extensions

---
