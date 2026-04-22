# ADR-003: Use mTLS for Local Communication

## Status
Accepted

## Context
Current communication is plaintext TCP, which is insecure and lacks authentication. As a security tool, it must follow Zero-Trust principles to prevent unauthorized command execution or status spoofing.

## Decision
Implement Mutual TLS (mTLS) using the `rustls` library.
1. Use a local Certificate Authority (CA) as the Root of Trust.
2. Issue unique certificates to the Server and Client, signed by the CA.
3. Configure the Server to require client authentication during the TLS handshake.
4. Use the Ed25519 elliptic curve for high performance and strong security.

## Consequences
- **Security**: Ensures end-to-end encryption and strong identity verification.
- **Robustness**: Connections from unauthorized parties are dropped during the handshake.
- **Maintenance**: Requires a mechanism to distribute CA certificates and handle certificate rotation.
