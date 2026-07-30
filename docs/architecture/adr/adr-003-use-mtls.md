# ADR-003: Use mTLS for Local Communication

## Status

Rejected ❌

## Context

The original proposal was to use mutual TLS (mTLS) to encrypt and authenticate communication between the client and server components.

The current implementation communicates over localhost TCP (port 50505) with plaintext JSON payloads. Security relies on:

- Binding to localhost only (127.0.0.1) — no exposure to external networks
- OS-level access controls on the local machine

## Decision

**Rejected.** After evaluation, mTLS is not needed because:

1. **Both client and server run on the same machine** — all communication is strictly localhost with no external network exposure.
2. **No external services use this protocol** — the TCP channel is only used between the local Tauri client and the local Rust server.
3. **Adding mTLS would introduce certificate management complexity** (generation, rotation, storage) with no meaningful security benefit for a local-only interface.
4. **The risk surface** is limited to other processes running on the same machine, which would already have access to the user's session and files.

The current approach of binding to localhost and relying on OS access controls is considered sufficient for this architecture.

## Consequences

- ✅ Simpler implementation — no certificate generation or management needed
- ✅ Easier debugging and development — plaintext communication on localhost
- ✅ No certificate rotation or expiry concerns
- ✅ No additional dependency (rustls, openssl) for the communication layer
- ⚠️ Any local process on the same machine can connect to the server (limited by OS-level controls)
- ⚠️ No encryption of data in transit between client and server (acceptable for localhost)
