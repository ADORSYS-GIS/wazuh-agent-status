# Go → Rust Migration Plan (wazuh-agent-status)

## Status (rolling)

- ✅ Implemented: protobuf contract, gRPC daemon skeleton, Unix/TCP transports, basic tray polling + actions, initial installer wiring.
- ⚠️ Not yet validated: running `cargo test` / `cargo build` end-to-end, and runtime behavior on each OS.

## Constraints (locked in)

- **Transport**
  - macOS + Linux: **gRPC over Unix domain socket**.
  - Windows: **gRPC over TCP loopback only** (`127.0.0.1`), never bind `0.0.0.0` or any non-loopback interface.
- **Daemon privileges**
  - Linux/macOS daemon runs as **`root:wazuh`** (root user, `wazuh` group).
  - Windows service runs as **LocalSystem** (or a dedicated service user if required later).
- **Version check**
  - Keep the existing `versionURL` behavior (GitHub raw file).

## Current-state summary (for parity)

- Go daemon (`wazuh-agent-status/`) exposes a simple TCP text protocol on port `50505` with commands:
  - `status` → `(agent status, connection status)`
  - `pause`
  - `restart`
  - `update` + `update-status`
  - `check-version` → compares local version file vs `versionURL`
- Go tray (`wazuh-agent-status-client/`) uses systray and polls every few seconds, enabling/disabling menu items based on status/version/update state.
- `scripts/install.sh` already installs both apps and configures “start at launch”:
  - Linux: systemd service + XDG autostart `.desktop`
  - macOS: LaunchDaemon + LaunchAgent

## Target architecture (Rust workspace)

- `crates/wazuh-status-proto-build`: protobuf compilation via `tonic-build`.
- `crates/wazuh-status-socket`: transport helpers:
  - `UnixSocketServer/Client` for macOS/Linux
  - `TcpLoopbackServer/Client` for Windows (hard-coded to `127.0.0.1`)
- `crates/wazuh-status-core`: OS-specific “agent control” implementation.
- `crates/wazuh-status-daemon`: privileged daemon binary exposing gRPC service.
- `crates/wazuh-status-common`: shared types/config (paths, constants).
- `crates/wazuh-status-client/src-tauri`: tray-first Tauri client using Rust gRPC client.

## Phase 0 — Decisions & acceptance criteria

### Acceptance criteria (end-to-end)

1. Daemon starts at boot:
   - Linux: systemd unit runs as `root`, group `wazuh`, creates socket with `0660` and `wazuh` group access.
   - macOS: LaunchDaemon runs as root; socket owned by `root:wazuh` with `0660`.
   - Windows: service listens on `127.0.0.1:<port>` only.
2. Client starts at login:
   - Linux: autostart entry.
   - macOS: LaunchAgent.
   - Windows: startup entry (or scheduled task / registry run key).
3. Tray app works without a visible window:
   - app launches and shows tray icon + menu, even if no window is created/shown.
4. Feature parity with Go:
   - Status + connection updates.
   - Pause, restart, update actions.
   - Update-in-progress feedback.
   - Version check against `versionURL`.

### Explicit security requirements

- Windows: enforce **loopback binding**; reject/ignore configuration that attempts to bind to non-loopback.
- Linux/macOS: prefer UDS path in a directory that can be owned `root:wazuh` and set to `0770` (directory) + `0660` (socket).

## Phase 1 — Define the protobuf contract first

### Deliverables

- ~~Add `proto/wazuh_status.proto` (or equivalent) defining the `WazuhStatus` service + messages.~~
- ~~Wire Rust type generation via `tonic-build` (build script).~~
- TODO: run/verify the generated code compiles on all targets (some issues only appear after `cargo build`).

### Notes

- Keep message shapes explicit and stable (avoid stringly-typed “Status: …”).
- Include structured fields:
  - `agent_state`: `ACTIVE | INACTIVE | UNKNOWN`
  - `connection_state`: `CONNECTED | DISCONNECTED | UNKNOWN`
  - `version_state`: `UP_TO_DATE | OUTDATED | UNKNOWN`
  - `version`: string (optional, when known)
  - `update_state`: `IDLE | IN_PROGRESS | FAILED | UNKNOWN`

## Phase 2 — Transport layer (Unix socket + Windows loopback TCP)

### Deliverables

- ~~`wazuh-status-socket` exposes a shared bind/connect API (`bind_incoming`, `connect_channel`).~~
- ~~macOS/Linux: bind UDS, clean up stale socket, set socket dir `0770` and socket `0660`, `chown root:wazuh` when running as root.~~
  - Note: `chown root:wazuh` only applies if effective uid is `0`; otherwise it leaves ownership unchanged (dev-friendly).
- ~~Windows: bind TCP to `127.0.0.1:<port>` only.~~
  - Comment: this effectively enforces loopback binding by construction; if we later add a configurable bind address, we must add an explicit “must be loopback” validator.

### Acceptance checks

- TODO: add explicit guardrails if we ever allow a bind address (today we hard-bind `127.0.0.1`).
- TODO: validate on Windows that the service is not reachable remotely (expected due to loopback binding).

## Phase 3 — Core agent control (port Go OS logic into Rust)

### Deliverables

- ~~Implemented core functions in `wazuh-status-core` for status/pause/restart/update/version (function-based, not a trait yet).~~
- ~~Linux/macOS: uses `wazuh-control` + reads `wazuh-agentd.state`.~~
- ~~Windows: uses PowerShell service control + reads `wazuh-agent.state`.~~
- TODO: make the core module more testable (abstract command execution / file reads) and add unit tests for parsers/decisions.

### Privilege strategy

- The daemon is privileged (root/system), so avoid interactive `sudo` prompts.
- Keep “dev mode” optional if needed (but do not rely on it in production).

## Phase 4 — Daemon (gRPC server + state machine)

### Deliverables

- ~~Daemon exposes gRPC service and basic update state machine.~~
- ~~CLI config exists: `--socket-path` (Unix) and `--port` (Windows).~~
- TODO: add `tracing` + file appender and align log paths with requirements.
- TODO: add graceful shutdown and cleanup behavior (signal handling).

### Acceptance checks

- TODO: verify `just run-daemon` end-to-end (daemon starts, client can query).
- TODO: verify Windows service binds only to `127.0.0.1` (expected).

## Phase 5 — Client library (shared gRPC wrapper)

### Deliverables

- ~~Created shared Rust client wrapper crate (`wazuh-status-common`) and wired it into Tauri.~~

### Why this helps

- It decouples tray UI work from transport/service work and speeds up debugging.

## Phase 6 — Tauri tray-first client (no-window requirement)

### Deliverables

- ~~Tray menu + polling/actions wired in Rust (Status/Connection/Version/Update/Pause/Restart/Quit).~~
  - Comment: we still need to add menu icons (green/gray) and handle “daemon unreachable” states more gracefully.
  - Comment: current Quit uses `std::process::exit(0)`; switch to `app.exit(0)` for cleaner teardown.
- TODO: enforce “tray-only by default” behavior (no visible window on start) while keeping the window available for display.
- TODO: throttle/merge polling so we don’t create many new gRPC connections too frequently (introduce a client wrapper and reuse a channel when possible).

### Acceptance checks

- TODO: confirm `just run-status` shows only tray icon by default (adjust `tauri.conf.json`/window creation as needed).

## Phase 7 — Startup/installer integration (root:wazuh, start at launch)

### Linux

- ~~Updated `scripts/install.sh` to create the socket dir and set systemd unit `User=root` + `Group=wazuh` and pass `--socket-path`.~~
- TODO: confirm the installed daemon binary name and arguments match the Rust binary naming scheme we will ship (post-Go cutover).

### macOS

- ~~Updated `scripts/install.sh` to ensure socket dir `root:wazuh` and pass `--socket-path` to LaunchDaemon.~~
- TODO: validate that the LaunchDaemon plist `GroupName` injection is correct (it’s currently generated inline in the heredoc).

### Windows

- ~~Updated `scripts/install.ps1` to start the daemon with `--port 50505` as a LocalSystem service and keep tray autostart.~~
- TODO: optionally add a firewall rule or explicit “deny non-loopback” check if we ever allow overriding bind address (currently we hard-bind `127.0.0.1` in Rust).

## Phase 8 — Cutover strategy

- Ship Rust daemon + Rust tray behind a “beta” channel for 1–2 releases.
- Keep Go binaries available for rollback during the transition.
- Once parity + startup behavior are stable across OSes, remove Go code and simplify scripts.

## Suggested execution order (work breakdown)

1. ~~Protobuf + generated code wired into workspace.~~
2. ~~Transport helpers (UDS + Windows loopback TCP).~~
3. ~~Port agent control logic per OS into `wazuh-status-core`.~~
4. ~~Wire full gRPC service + basic update state machine.~~
5. ~~Implement tray menu + polling/actions in Tauri (partial; tray-only-by-default still pending).~~
6. ~~Add Rust client wrapper (shared by Tauri and tooling).~~
7. Next: add tracing/logging + graceful shutdown.
8. Next: verify installers on all OSes; finalize `root:wazuh` and loopback-only guarantees.
