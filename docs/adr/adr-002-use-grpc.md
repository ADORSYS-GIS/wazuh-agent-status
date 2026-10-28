# ADR-002: Use gRPC for Communication

## Status

Deferred ⏳

## Context

The current implementation uses a custom TCP-based protocol over localhost (port 50505). Commands are sent as newline-terminated plaintext strings (e.g., `get-version`, `subscribe-status`, `initiate-update-stream`), and responses are returned as plaintext with structured JSON payloads.

While this works, it has limitations:

- No formal schema for requests/responses
- No built-in streaming contract (status subscriptions are handled manually)
- No encryption (relies on localhost binding for security)
- Manual serialization/deserialization of JSON payloads

## Decision

**Deferred.** A migration to gRPC was considered but is not yet implemented.

The current Rust server uses a custom TCP protocol over localhost with JSON payloads. This was chosen for simplicity during the initial Rust migration. Future work may adopt gRPC for structured communication between client and server.

## Consequences

- ⏳ TCP protocol remains in use for now
- ⏳ No schema enforcement at the wire level
- ⏳ Client and server must agree on message formats manually
- ✅ Future migration to gRPC is simplified by the existing trait-based abstraction layers
- ✅ gRPC would bring protobuf schemas, bidirectional streaming, and built-in TLS support
