# ADR-0007: Automate Dependency Updates with Dependabot

> We introduce GitHub Dependabot configuration across all tracked package ecosystems (Cargo, npm, Bundler, and GitHub Actions) to automate dependency updates with pull request limits and cool-down buffers.

## Metadata

| Field             | Value                                              |
| :---------------- | :------------------------------------------------- |
| **Document ID**   | ADR-0007                                           |
| **Status**        | `Accepted`                                         |
| **Date**          | 2026-11-06                                         |
| **Deciders**      | Wazuh maintainers                                  |
| **Governs**       | `.github/dependabot.yml`                           |
| **Amends**        | N/A                                                |
| **Amended by**    | N/A                                                |
| **Supersedes**    | N/A                                                |
| **Superseded by** | N/A                                                |

## Context

The `wazuh-agent-status` repository comprises several sub-projects spanning multiple distinct package managers and ecosystems:
1. **Rust / Cargo**: The desktop status client backend (`wazuh-agent-status-rust-client/src-tauri/Cargo.toml`) and the local status server (`wazuh-agent-status-rust-server/Cargo.toml`).
2. **JavaScript / npm**: The React frontend (`wazuh-agent-status-rust-client/package.json`) and associated UI dependencies.
3. **Ruby / Bundler**: The Jekyll-based documentation site (`docs/Gemfile`).
4. **CI/CD Actions**: Workflows in `.github/workflows/` (`actions/checkout`, `SonarSource/sonarqube-scan-action`, CodeQL, etc.).

Previously, dependency updates across these four distinct ecosystems were coordinated manually or via ad-hoc Dependabot runs without standardized configuration. This created risks of version drift (such as mismatches between Tauri JS packages and Rust crates, as addressed in PR #278), unpatched security advisories, and runner runtime deprecations.

Under Epic ADORSYS-GIS/wazuh#2354, all actively maintained repositories in the ADORSYS-GIS organization are standardized on a common Dependabot configuration pattern modeled after `wazuh-cert-oauth2`.

The deciding forces are:
- Automatically identify and patch dependency vulnerabilities across all four language/manifest ecosystems.
- Synchronize update cadences between related client packages.
- Throttling pull requests with concurrency limits and cool-down periods to keep maintainer review queues manageable.

## Decision

We will configure GitHub Dependabot in `.github/dependabot.yml` covering all active package manifests:

1. **Tracked Ecosystems & Schedules**:
   - `cargo`:
     - Directory: `/wazuh-agent-status-rust-client/src-tauri`, schedule `daily` at `08:00` UTC.
     - Directory: `/wazuh-agent-status-rust-server`, schedule `daily` at `08:00` UTC.
   - `npm`:
     - Directory: `/wazuh-agent-status-rust-client`, schedule `daily` at `08:00` UTC.
   - `bundler`:
     - Directory: `/docs`, schedule `daily` at `08:00` UTC.
   - `github-actions`:
     - Directory: `/`, schedule `weekly` at `08:00` UTC.

2. **Throttling & Buffering**:
   - `open-pull-requests-limit: 10` per ecosystem.
   - `cooldown.default-days: 7` to allow upstream releases to stabilize before opening non-critical PRs.

## Options considered

| Option                                              | Verdict  | Why (one line)                                                                    |
| :-------------------------------------------------- | :------- | :-------------------------------------------------------------------------------- |
| Manual updates per manifest (status quo)            | Rejected | High maintenance burden and frequent package synchronization drift.               |
| Separate configuration per component                | Rejected | Fragmented maintenance; a unified repository configuration is cleaner.            |
| Unified multi-ecosystem Dependabot config (Adopted) | Adopted  | Standardizes all 4 ecosystems under the org convention with native GitHub tooling.|

## Consequences

**Not part of this decision:** Automated merge workflows. All opened pull requests must pass standard CI (CodeQL, SonarQube, SAST, unit tests) and receive code review before merging.

### Impact

**Benefits**

- Continuous scanning and automated patching across Cargo, npm, Bundler, and GitHub Actions.
- Controlled volume via 10 PR caps and 7-day cooldowns.
- Reduced risk of build failures from out-of-sync cross-ecosystem dependencies.

**Costs**

- Maintainers will review incoming dependency PRs as part of regular sprint activities.

### Done when

- [x] `.github/dependabot.yml` covers Cargo (client and server), npm, Bundler, and GitHub Actions.
- [x] Schedules, open PR caps, and cooldowns adhere to the organization standard.
- [x] ADR-0007 is documented and registered in `docs/adr/README.md`.

## Rollback

Delete or revert `.github/dependabot.yml`. Dependabot will cease opening automated update pull requests.

## Evidence

| Claim                                                                 | Where                                                          |
| :-------------------------------------------------------------------- | :------------------------------------------------------------- |
| Organization reference convention                                     | `ADORSYS-GIS/wazuh-cert-oauth2/.github/dependabot.yml`         |
| Sub-issue scope definition                                            | `ADORSYS-GIS/wazuh-agent-status#276`                           |
| Organization epic tracking                                            | `ADORSYS-GIS/wazuh#2354`                                       |

## References

- [GitHub Dependabot Configuration Options](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-options-for-the-dependabot.yml-file)
- [ADR template](https://github.com/ADORSYS-GIS/wazuh/blob/main/docs/adr/template.md)
- [ADR writing guide](https://github.com/ADORSYS-GIS/wazuh/blob/main/docs/adr/GUIDE.md)

## Solution implementation

| Kind   | Tracking                                                                                         | Status | What it covers                                       |
| :----- | :----------------------------------------------------------------------------------------------- | :----- | :--------------------------------------------------- |
| Issue  | [ADORSYS-GIS/wazuh-agent-status#276](https://github.com/ADORSYS-GIS/wazuh-agent-status/issues/276) | Open   | Add dependabot configuration to wazuh-agent-status   |
| PR     | [ADORSYS-GIS/wazuh-agent-status#282](https://github.com/ADORSYS-GIS/wazuh-agent-status/pull/282)   | Open   | Adds .github/dependabot.yml and ADR-0007             |
