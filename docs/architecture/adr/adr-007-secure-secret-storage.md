# ADR-007: Secure Secret Storage and Native Key Stores

## Status
Accepted

## Context
Private keys for mTLS are highly sensitive. Storing them as plain files in the filesystem, even with restricted permissions, can be a security bottleneck. Enterprise security standards often require the use of hardware-backed or OS-native secure storage.

## Decision
Abstract secret retrieval behind a `SecretStore` interface and implement support for native OS key stores.
1.  **Windows**: Integrate with the **Windows Certificate Store** (Personal Store).
2.  **macOS**: Use the **System Keychain**.
3.  **Linux**: Use restricted file permissions (`chmod 600`) or integration with **libsecret**.

## Consequences
- **Security**: Private keys are protected by OS-level access controls and potentially TPM/HSM hardware.
- **Manageability**: IT administrators can manage certificates using standard enterprise tools (e.g., Group Policy, MDM).
- **Complexity**: Increases implementation effort and platform-specific code.
