# Go → Rust Migration Plan (wazuh-agent-status)

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

- Add `proto/wazuh_status.proto` (or equivalent) defining:
  - `service WazuhStatus`
    - `rpc GetStatus(Empty) returns (StatusReply)`
    - `rpc Pause(Empty) returns (ActionReply)`
    - `rpc Restart(Empty) returns (ActionReply)`
    - `rpc StartUpdate(Empty) returns (ActionReply)`
    - `rpc GetUpdateStatus(Empty) returns (UpdateStatusReply)`
    - `rpc CheckVersion(Empty) returns (VersionReply)`
- Generate Rust types + server/client stubs via `tonic-build`.

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

- `wazuh-status-socket` exposes a single “dial” API the client and daemon can share, e.g.:
  - `ServerBind::bind(config) -> impl Stream<IO>`
  - `ClientConnect::connect(config) -> tonic::transport::Channel`
- macOS/Linux:
  - Use `tokio::net::UnixListener` + `tonic::transport::Server::builder().serve_with_incoming(...)`.
  - Ensure socket file cleanup on startup, and `chmod/chown` to `root:wazuh` (`0660`).
- Windows:
  - Use `tokio::net::TcpListener` binding to `127.0.0.1:<port>`.
  - Hard-check that the resolved bind address is loopback (not `0.0.0.0`).

### Acceptance checks

- Attempting to bind to `0.0.0.0` should fail fast with a clear error.
- A remote host should not be able to connect on Windows (loopback-only binding).

## Phase 3 — Core agent control (port Go OS logic into Rust)

### Deliverables

- `wazuh-status-core` implements a trait like:
  - `check_service_status() -> (AgentState, ConnectionState)`
  - `pause_agent()`
  - `restart_agent()`
  - `update_agent()`
  - `get_local_version() -> Option<String>`
  - `fetch_online_version(version_url) -> Option<String>`
- OS-specific backends:
  - Linux/macOS: call `wazuh-control` and read `wazuh-agentd.state` equivalent.
  - Windows: service control + state file equivalent using PowerShell or native APIs (start with PowerShell parity, then move to Windows APIs if needed).

### Privilege strategy

- The daemon is privileged (root/system), so avoid interactive `sudo` prompts.
- Keep “dev mode” optional if needed (but do not rely on it in production).

## Phase 4 — Daemon (gRPC server + state machine)

### Deliverables

- `crates/wazuh-status-daemon/src/main.rs` becomes:
  - CLI config (`--socket-path` for Unix; `--listen-port` for Windows).
  - gRPC service implementation.
  - `update_state` stored in a shared state object (e.g., `Arc<Mutex<...>>`), with update running in a background task.
  - logging via `tracing` + file appender (Linux `/var/log/...`, macOS `/var/log/...` or `/Library/Logs/...`).

### Acceptance checks

- `just run-daemon` works locally using `/tmp/wazuh-status-daemon-dev.sock` (macOS/Linux).
- On Windows dev: daemon binds only to `127.0.0.1`.

## Phase 5 — Client library + CLI (optional but recommended)

### Deliverables

- A small Rust client wrapper crate (could live in `wazuh-status-common` or a new crate) that:
  - dials UDS or TCP based on OS
  - exposes methods matching the gRPC API
- Optional: a `wazuh-status` CLI binary for quick testing:
  - `wazuh-status status|pause|restart|update|version`

### Why this helps

- It decouples tray UI work from transport/service work and speeds up debugging.

## Phase 6 — Tauri tray-first client (no-window requirement)

### Deliverables

- Update `crates/wazuh-status-client/src-tauri/src/lib.rs`:
  - Build tray menu mirroring Go systray behavior (status lines + actions).
  - Poll daemon on a timer (e.g., every 5s for status; every 4h for version), but avoid duplicate concurrent polling.
  - Menu actions call gRPC methods.
- Keep the window config for “display only”, but ensure:
  - app can run with **no window shown** by default
  - tray remains functional without creating/focusing a window

### Acceptance checks

- Running `just run-status` shows only tray icon (no visible window) unless explicitly opened.

## Phase 7 — Startup/installer integration (root:wazuh, start at launch)

### Linux

- Update `scripts/install.sh` to:
  - install Rust daemon + client binaries
  - systemd unit:
    - `User=root`
    - `Group=wazuh`
    - ensure socket directory exists and permissions are correct
  - XDG autostart entry points to the tray binary

### macOS

- Update `scripts/install.sh` to:
  - LaunchDaemon for daemon (runs as root) + set group to `wazuh` if required by creating/chowning socket directory at install time
  - LaunchAgent for tray app

### Windows

- Update `install.ps1` to:
  - install daemon as service (LocalSystem)
  - configure tray app to start at login (startup folder / registry run key / scheduled task)
  - enforce daemon listen address `127.0.0.1` in config defaults

## Phase 8 — Cutover strategy

- Ship Rust daemon + Rust tray behind a “beta” channel for 1–2 releases.
- Keep Go binaries available for rollback during the transition.
- Once parity + startup behavior are stable across OSes, remove Go code and simplify scripts.

## Suggested execution order (work breakdown)

1. Protobuf + generated code wired into workspace.
2. Transport helpers (UDS + Windows loopback TCP).
3. Daemon skeleton + health endpoints (GetStatus returns placeholder).
4. Port agent control logic per OS into `wazuh-status-core`.
5. Wire full gRPC service + update state machine.
6. Add Rust client wrapper + optional CLI.
7. Implement tray menu + polling/actions in Tauri (tray-only by default).
8. Update install/startup scripts for all OSes; verify `root:wazuh` on macOS/Linux and loopback-only on Windows.

