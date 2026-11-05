# Architecture Decision Records

Every architectural decision is recorded here as a numbered ADR: one file per
decision. The canonical writing guidance is the central ADR guide, hosted in the
central documentation repository:
<https://github.com/ADORSYS-GIS/wazuh/blob/main/docs/adr/GUIDE.md>
(read it before writing an ADR).

## Index

| ID                                             | Title                                               | Status   | Date       | Relations           |
| :--------------------------------------------- | :-------------------------------------------------- | :------- | :--------- | :------------------ |
| [ADR-0001](0001-use-rust.md)                   | Use Rust for Client and Server                      | Accepted | 2026-04-01 | N/A                 |
| [ADR-0002](0002-use-grpc.md)                   | Use gRPC for Communication                          | Deferred | 2026-04-01 | N/A                 |
| [ADR-0003](0003-use-mtls.md)                   | Use mTLS for Local Communication                    | Rejected | 2026-04-01 | N/A                 |
| [ADR-0004](0004-rust-migration-complete.md)    | Rust Migration Complete — Go Implementation Removed | Accepted | 2026-06-30 | relates to ADR-0001 |
| [ADR-0005](0005-generate-sbom-in-ci.md)        | Generate CycloneDX SBOM in CI                       | Accepted | 2026-10-20 | N/A                 |
| [ADR-0006](0006-unify-sca-compliance-views.md) | Unify SCA Compliance Views                          | Accepted | 2026-11-05 | N/A                 |
