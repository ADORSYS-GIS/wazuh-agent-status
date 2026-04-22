# ADR-002: Use mTLS-Secured Command Protocol

## Status
Accepted

## Context
The application requires a way to communicate commands and status updates between the tray client and the background server. Initial designs considered gRPC, but for local, low-latency, and highly secure communication, a simpler protocol over mTLS is more efficient.

## Decision
Use a lightweight, line-based command protocol secured by Mutual TLS (mTLS) instead of gRPC.

## Consequences
- **Efficiency**: Lower overhead than gRPC for simple command sets.
- **Security**: Strong encryption and authentication provided by the mTLS layer.
- **Simplicity**: Easier to implement and debug without complex protobuf dependencies.
