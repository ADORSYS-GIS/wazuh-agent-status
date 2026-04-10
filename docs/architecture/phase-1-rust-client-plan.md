# Implementation Plan: Phase 1 (Rust Tray Client)

## Goal

Build a high-performance, cross-platform system tray application in Rust that replaces the current Go client while maintaining compatibility with the existing Go backend.

---

## 🛠️ Step 1: Project Initialization

- Create a new directory: `wazuh-agent-status-rust-client`.
- Initialize `cargo init`.
- Configure `Cargo.toml` with dependencies: `tray-icon`, `tokio`, `anyhow`, `rust-embed`.

## 📡 Step 2: Compatibility Layer (The "Bridge")

- Implement a simple TCP client using `tokio::net::TcpStream`.
- Create a function to send the `"status\n"` command and parse the plaintext response.
- **Goal**: Ensure the Rust client can talk to the default Go server on port 50505.

## 🎨 Step 3: Tray UI Implementation

- Use the `tray-icon` crate for cross-platform support.
- Implement the read-only status menu:
  - 📡 **Agent Status** (e.g., "Active")
  - 🔌 **Connection** (e.g., "Connected")
  - ℹ️ **Version** (e.g., "1.8.x")
  - ⚠️ **Status**: "Outdated" (if local version is old)
  - 🚀 **Update to Stable** (clickable if outdated)
  - ✨ **Update to Prerelease** (visible if preview available & agent in test group)
  - ❌ **Quit**
- Use `rust-embed` to pack existing icons/assets into the binary.

## 🔄 Step 4: Async Event Loop

- Implement a `tokio` polling loop that pings the server every 5 seconds (matching current behavior).
- Ensure the UI remains responsive during network timeouts.

## 📊 Step 5: Benchmark & Verification

- Compile the Rust client in `--release` mode.
- Run side-by-side with the Go client.
- **Success Criteria**:
  - RAM usage < 10MB (compared to ~30-50MB).
  - CPU usage remains < 1% during idle.
  - Zero functional regression (Restart/Update/Prerelease-Update still work).

---

## 🏗️ Future-Proofing

- Ensure the connection logic is behind a trait (interface) so we can swap it for **gRPC** easily in Phase 2.
