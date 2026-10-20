# ADR-005: Generate SBOM in CI

## Status

Accepted ✅

## Context

This project is a desktop agent status client and server built with Rust, a
React frontend, and Ruby documentation. It needs an accurate inventory of
everything it delivers so that the supply chain can be audited.

The GitHub Dependency Graph was enabled as the first option. It is free and
needs no setup, but it produced a minimal SBOM for this repository. It only
covers the package ecosystems GitHub parses, and it only reads committed
manifests. Dependencies that are resolved at build time, or that come from
ecosystems GitHub does not support, are missing from the graph.

The dependency graph is therefore not a reliable source for what this
project actually ships.

## Decision

Generate a CycloneDX SBOM in CI with Trivy, using the shared `sbom.yml`
workflow from [adorsys-ci-cd-templates](https://github.com/ADORSYS-GIS/adorsys-ci-cd-templates). The job runs from the SAST workflow
[sast.yaml](../../../.github/workflows/sast.yml), scans the repository contents, and uploads
`sbom.json` as a workflow artifact.

The GitHub Dependency Graph stays enabled as a baseline and for Dependabot
alerts, but the CI-generated SBOM is the source of truth.

## Consequences

- ✅ The SBOM reflects the repository contents as scanned by Trivy, including
  dependencies the dependency graph cannot see
- ✅ The SBOM is produced on every push to the default branch and on pull
  requests
- ✅ The same workflow is reused across all Wazuh projects, so results are
  consistent
- ✅ The artifact can be imported into CycloneDX tooling or DefectDojo
- ⚠️ The SBOM is a snapshot of the repository at scan time, not of a built
  artifact
- ⚠️ The reusable workflow reference must be pinned to a commit SHA once the
  shared workflow is merged
