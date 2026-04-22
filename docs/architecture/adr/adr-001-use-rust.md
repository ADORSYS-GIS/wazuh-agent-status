# ADR-001: Use Rust for Agent Implementation

## Status
Accepted

## Context
The legacy Wazuh agent status components (e.g., in Go) often suffer from high memory footprints and complex cross-compilation due to CGO dependencies. We need a performant, memory-safe, and lightweight solution that can run reliably on diverse endpoints (Windows, Linux, macOS).

## Decision
Use Rust for both the background server and the system tray application.

## Consequences
- **Memory Safety**: Eliminates entire classes of memory bugs (buffer overflows, etc.).
- **Performance**: Near-native performance with minimal overhead.
- **Simplified Deployment**: Static binaries with no runtime dependencies, facilitating easy distribution.
- **Developer Productivity**: Higher learning curve initially, but better reliability in production.
