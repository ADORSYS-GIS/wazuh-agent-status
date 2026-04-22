# ADR-005: Use Asynchronous I/O with Tokio

## Status
Accepted

## Context
The application needs to handle multiple concurrent client connections and background monitoring tasks without blocking the main thread, especially on resource-constrained machines.

## Decision
Use the `tokio` asynchronous runtime for all I/O operations (TCP, file polling, and timers).

## Consequences
- **Performance**: Efficiently manages thousands of concurrent tasks with minimal thread overhead.
- **Responsiveness**: The system tray UI remains responsive even during heavy server load.
- **Maintainability**: Leverages the mature Rust async ecosystem.
