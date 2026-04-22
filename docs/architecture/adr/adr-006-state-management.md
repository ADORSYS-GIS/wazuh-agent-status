# ADR-006: State Management with Broadcast and Watch Channels

## Status
Accepted

## Context
The `AgentManager` needs to push real-time status updates to multiple connected TCP clients and the local UI concurrently. Traditional polling or single-consumer channels are insufficient for one-to-many state propagation.

## Decision
Use `tokio::sync::watch` for local state tracking (latest status) and `tokio::sync::broadcast` for pushing updates to remote TCP clients.

## Consequences
- **Efficiency**: Clients only receive updates when a change actually occurs, reducing unnecessary I/O.
- **Consistency**: New clients immediately receive the latest state upon connection via the `watch` channel.
- **Decoupling**: The monitoring logic is completely separated from the communication and UI layers.
