# Architecture Decision Records

Every architectural decision is recorded here as a numbered ADR: one file per
decision. The canonical writing guidance is the central ADR guide, hosted in the
central documentation repository:
<https://github.com/ADORSYS-GIS/wazuh/blob/main/docs/adr/GUIDE.md>
(read it before writing an ADR).

## Index

| ID      | Title                                               | Status   | Date | Relations          |
| :------ | :-------------------------------------------------- | :------- | :--- | :----------------- |
| [ADR-001](adr-001-use-rust.md) | Use Rust for Client and Server                      | Accepted | N/A  | N/A                |
| [ADR-002](adr-002-use-grpc.md) | Use gRPC for Communication                          | Deferred | N/A  | N/A                |
| [ADR-003](adr-003-use-mtls.md) | Use mTLS for Local Communication                    | Rejected | N/A  | N/A                |
| [ADR-004](adr-004-rust-migration-complete.md) | Rust Migration Complete — Go Implementation Removed | Accepted | N/A  | relates to ADR-001 |
| [ADR-005](adr-005-generate-sbom-in-ci.md) | Generate CycloneDX SBOM in CI                       | Accepted | N/A  | N/A                |
