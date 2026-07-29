---
layout: default
title: "ADR-001: Use Rust for Client and Server"
parent: Architecture & ADRs
---

# ADR-001: Use Rust for Client and Server

## Status

Accepted 

## Context

The original Go implementation had high memory usage, CGO dependency overhead, and limited cross-platform consistency. The Go client consumed ~10 MB RSS with ~2.2 GB VSZ, and the Go server consumed ~10.5 MB RSS with ~1.9 GB VSZ.

The system required a more performant, secure, and maintainable implementation.

## Decision

Use Rust for both the tray client and the background status server.

- **Client**: Built with Tauri (Rust backend + web frontend) for cross-platform system tray functionality.
- **Server**: Built with Tokio async runtime for efficient I/O, using TCP for client communication.
- **Communication**: TCP-based protocol (plaintext JSON commands) between client and server.

## Consequences

-  Better performance — significantly lower memory usage and faster execution
-  No CGO dependency — pure Rust compilation simplifies cross-compilation and deployment
-  Stronger security guarantees via Rust's memory safety
-  Cross-platform support (Linux, macOS, Windows) via a single codebase
-  More complex development initially, but improved maintainability over time

## Implementation Notes

- The Rust migration is complete as of v0.5.x.
- The Go implementation has been fully removed from the repository.
- Install scripts handle cleanup of legacy Go components.
