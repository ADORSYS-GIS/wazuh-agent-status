---
layout: default
title: "ADR-004: Rust Migration Complete — Go Implementation Removed"
parent: Architecture & ADRs
---

# ADR-004: Rust Migration Complete — Go Implementation Removed

## Status

Accepted 

## Context

The project initially had a Go-based implementation split across two components:

- `wazuh-agent-status-client/` — Go tray client using `fyne.io/systray`
- `wazuh-agent-status/` — Go TCP server using the standard `net` library

A decision was made in [ADR-001](adr-001-use-rust.md) to migrate the system to Rust for better performance, security, and maintainability. The migration was executed incrementally:

**Phase 1**: Build a Rust replacement client (`wazuh-agent-status-rust-client/` using Tauri) while keeping the Go server running.

**Phase 2**: Build a Rust replacement server (`wazuh-agent-status-rust-server/` using Tokio) with feature parity to the Go server.

**Phase 3**: Remove all Go code, verify feature parity, and update documentation.

## Decision

The migration is complete. All Go implementation files and directories have been removed from the repository. The system is now fully Rust-based:

| Component                   | Original (Go)                | Current (Rust)                            |
| --------------------------- | ---------------------------- | ----------------------------------------- |
| Client (Tray App)           | `wazuh-agent-status-client/` | `wazuh-agent-status-rust-client/` (Tauri) |
| Server (Background Service) | `wazuh-agent-status/`        | `wazuh-agent-status-rust-server/` (Tokio) |
| Communication               | TCP (plaintext)              | TCP (plaintext) — unchanged               |

The Go implementation is preserved for reference in the `legacy/go-implementation` branch.

### Feature Parity Verified

| Feature                                 | Go  | Rust     |
| --------------------------------------- | --- | -------- |
| Agent status monitoring                 |   |        |
| Real-time status updates (subscribe)    |   |        |
| Version checking & update notifications |   |        |
| Auto-update (stable + prerelease)       |   |        |
| Log streaming                           |   |  (new) |
| SCA Compliance dashboard                |   |  (new) |
| AI-powered remediation                  |   |  (new) |
| Cross-platform (Linux, macOS, Windows)  |   |        |

### Performance Improvements

| Metric         | Go (baseline) | Rust             |
| -------------- | ------------- | ---------------- |
| Server RSS     | ~10.5 MB      | < 5 MB           |
| Client RSS     | ~10.1 MB      | Varies (WebView) |
| Idle CPU       | < 0.1%        | < 0.5%           |
| CGO dependency | Yes           | None             |

## Consequences

-  Clean repository — only Rust code and associated scripts/docs remain
-  Simplified build process — Rust toolchain only (no Go SDK requirement)
-  Improved cross-platform consistency via single codebase
-  Better long-term maintainability with Rust's type system and memory safety
-  Team needs Rust proficiency (mitigated by training and documentation)
-  Go implementation available only in legacy branch for historical reference
