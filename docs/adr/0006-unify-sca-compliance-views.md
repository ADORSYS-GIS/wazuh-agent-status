# ADR-005: Unify SCA Compliance Views

## Status

Proposed 📝

## Context

### Problem

We currently have two separate views of Security Configuration Assessment (SCA) results:

1. **Agent Status Desktop Client** — shows a curated corporate compliance view with aggregated scores, custom categories, mandatory flags, and remediation instructions. Only accessible to users with the desktop application installed.

2. **Wazuh Dashboard** (web, deployed via Helm) — exposes native SCA results without corporate compliance enrichment. Shows raw check results per agent per policy, but lacks aggregated compliance scoring, mandatory flag visibility, and business-aligned category groupings.

SOC analysts need a centralized compliance view accessible from the web dashboard — to see compliance posture across all agents, filter by non-compliant agents, and understand which corporate security requirements are not met. The current split forces cross-referencing between two systems, and the corporate compliance score (the one that matters to the business) is only visible on the desktop client.

### Why the Results Differ

The `wazuh-gateway` fetches raw SCA results from the Wazuh Manager API and evaluates them against **corporate compliance profiles** (`linux.json`, `macos.json`, `windows.json`). These profiles are a curated subset of Wazuh's full SCA checks, adding:

| Dimension      | Wazuh Dashboard (Native)         | Agent Status Desktop Client                      |
| :------------- | :------------------------------- | :----------------------------------------------- |
| Data scope     | All checks from all SCA policies | Curated subset matching corporate policies       |
| Score          | None                             | `passed / (passed + failed) × 100`               |
| Categories     | Policy-level grouping only       | Business-aligned (Firewall, SSH Hardening, etc.) |
| Mandatory flag | Not supported                    | Per-check mandatory flag                         |
| Remediation    | From SCA policy YAML             | Custom corporate instructions                    |

### Current Data Flow

```
┌─────────────────────────────────────────────────────────┐
│               CURRENT ARCHITECTURE                       │
└─────────────────────────────────────────────────────────┘

  Wazuh Agent ──► Wazuh Manager ──► OpenSearch
                       │                │
                       │                ▼
                       │        ┌──────────────────┐
                       │        │  Wazuh Dashboard │
                       │        │  (native SCA)    │
                       │        └──────────────────┘
                       │
                       │ HTTP API
                       │ (/sca/*)
                       ▼
              ┌──────────────────┐
              │  wazuh-gateway   │
              │  (Compliance    │
              │   evaluation)    │
              └────────┬─────────┘
                       │
                       ▼
              ┌──────────────────┐
              │  Desktop Client  │
              │  (only consumer) │
              └──────────────────┘
```

The compliance evaluation logic exists only in the gateway and is consumed by a single client. No other system can access the corporate compliance view.

---

## Decision Drivers (Why are we making this change?)

- Maintain a **single compliance evaluation engine** — avoid duplicating corporate policy rules across consumers
- Allow SOC teams to consume compliance information **without desktop client access**
- Preserve compatibility with existing Wazuh SCA capabilities
- Minimize additional operational components
- Support future compliance consumers (API integrations, reports, SIEM, ticketing systems)

## Non-Goals

- Replacing Wazuh native SCA functionality
- Modifying Wazuh SCA policy files
- Building a new compliance evaluation engine (reuse existing)
- Creating a new authentication mechanism for compliance data
- Providing real-time compliance monitoring (periodic sync is sufficient)

---

## Decision

**The `wazuh-gateway` becomes the compliance evaluation and aggregation layer between Wazuh native SCA data and compliance consumers. Compliance evaluation and business rule interpretation belong to the gateway domain layer, not to individual consumers.**

### Architectural Change

```
┌─────────────────────────────────────────────────────────┐
│                 PROPOSED ARCHITECTURE                     │
└─────────────────────────────────────────────────────────┘

              ┌──────────────────┐
              │  Wazuh Manager   │
              │  (SCA data)      │
              └────────┬─────────┘
                       │
                       │ HTTP API (/sca/*)
                       │
             ┌─────────▼─────────┐
             │  Compliance Engine │
             │  (wazuh-gateway)  │
             │                   │
             │  - Fetches SCA    │
             │    from Manager   │
             │  - Evaluates      │
             │    corporate      │
             │    profiles       │
             └────────┬──────────┘
                      │
                 Compliance Model
                      │
        ┌─────────────┴─────────────┐
        │                           │
        v                           v

  OpenSearch Read Model        Gateway API
        │                           │
        v                           v

  Web Dashboard               Desktop Client
  Reports                     Future Apps
```

The gateway evolves from a single-consumer API to a **compliance evaluation and aggregation layer**. It produces a **Compliance Model** — the normalized domain representation containing compliance scores, statuses, categories, mandatory requirements, remediation information, and metadata required by downstream consumers. Consumers access this model through appropriate interfaces: OpenSearch serves as a **compliance read model** optimized for dashboard queries, and the Gateway API serves desktop and programmatic consumers.

The authoritative inputs are Wazuh SCA results and corporate compliance profiles. The Compliance Model generated by the gateway is the canonical representation of evaluated compliance posture consumed by downstream applications. OpenSearch is a projection, not the source of truth.

### Compliance Model Definition

The Compliance Model is the normalized representation of corporate compliance posture produced by the gateway after evaluating Wazuh SCA results against corporate compliance profiles. It contains:

- Agent compliance posture (compliant / non-compliant)
- Compliance score
- Mandatory requirement status
- Business-aligned categories (Firewall, SSH Hardening, etc.)
- Check-level compliance information
- Remediation guidance
- Profile version metadata

The Compliance Model is the contract between the compliance engine and its consumers. It exists independently of any specific storage or projection — OpenSearch documents and API responses are projections of this model.

### Why This Architecture

1. **Single evaluation engine** — Corporate compliance profiles (`linux.json`, `macos.json`, `windows.json`) drive all consumers. Updating a check's category or mandatory flag requires a single profile change, after which all consumers receive the updated evaluation result during the next synchronization cycle.

2. **No data drift** — The gateway is the only component that evaluates compliance. Consumers access the same evaluation output, so scores and statuses are consistent across all views.

3. **Decoupled consumers** — The solution separates compliance evaluation from compliance consumption by maintaining an optimized read model for dashboards. The web dashboard, desktop client, and future consumers are independent projections of the same compliance model. Adding a new consumer (e.g., a Slack notification, a Jira ticket creation, a PDF report) requires no changes to the evaluation logic.

4. **No new service** — The gateway already contains the compliance evaluation logic. Publishing to OpenSearch is an extension of an existing service, not a new deployment.

---

## Consequences

### Benefits

- ✅ A single compliance evaluation engine provides consistent results across all user interfaces
- ✅ Corporate compliance profiles are maintained in one place and consumed by all consumers
- ✅ SOC analysts access compliance data via the web dashboard without requiring desktop client installation
- ✅ Future consumers (API, reports, SIEM, ticketing) can be added without modifying the evaluation engine
- ✅ Minimal operational overhead — extends an existing service rather than introducing new components

### Trade-offs

- ⚠️ **Gateway scope creep** — The gateway now handles API traffic, compliance evaluation, batch synchronization, and OpenSearch indexing. At current scale this is acceptable. The compliance synchronization workflow should remain isolated behind a dedicated module boundary inside the gateway so that extraction into a separate worker remains possible without redesigning the domain logic. Extraction criteria include synchronization duration, API load on Wazuh Manager, or operational isolation requirements. Extraction should preserve the Compliance Model contract, allowing the evaluator to move without impacting consumers.
- ⚠️ **Gateway dependency** — If the gateway is down, the compliance index stops updating. Native SCA data in `wazuh-alerts-*` remains unaffected, and the dashboard's native SCA module continues to work.
- ⚠️ **Migration complexity** — Existing desktop clients continue using the gateway API directly while new consumers (web dashboard, future integrations) consume the enriched compliance projection. No breaking changes are introduced to the existing gateway API during the migration phase.
- ⚠️ **Periodic freshness** — Compliance data is as fresh as the last sync cycle. An on-demand refresh mechanism is available for time-sensitive scenarios.
- ⚠️ **Current posture only** — The current model provides current compliance posture only. Historical trend analysis (e.g., "were we compliant last month?") requires additional retention strategy.
- ⚠️ **OpenSearch storage** — Each agent × check generates a document. This is minimal for typical fleet sizes.

### Risk Mitigation

| Risk                           | Mitigation                                                                                              |
| :----------------------------- | :------------------------------------------------------------------------------------------------------ |
| Gateway downtime               | Indexing is idempotent; next successful cycle overwrites stale data. Native SCA data remains available. |
| Wazuh API rate limits at scale | Controlled parallel synchronization with configurable concurrency and exponential backoff.              |
| Stale data in dashboard        | On-demand refresh button for SOC analysts; scheduled synchronization keeps data fresh automatically.    |
| Profile changes                | Profiles are versioned JSON files; changes are auditable via git.                                       |

### Alternatives Considered

**Keep compliance only in desktop client.** Rejected because SOC users require web-based access without desktop client installation.

**Implement compliance logic inside OpenSearch dashboards.** Rejected because business rules would be duplicated in visualization tooling, creating maintenance burden and inconsistency risk.

**Create a dedicated compliance service.** Rejected at the current scale because introducing a dedicated service would duplicate deployment, monitoring, and operational responsibilities without providing immediate architectural benefit. The gateway boundaries remain designed so the evaluation component can be extracted later if scaling requirements justify it.

---

## References

- [Wazuh SCA Documentation](https://documentation.wazuh.com/current/user-manual/capabilities/sec-config-assessment/how-it-works.html)
- [wazuh-gateway SCA Doc](../../wazuh-gateway/doc/sca.md)
- [ADR-001: Use Rust](adr-001-use-rust.md)
- [Compliance Read Model Design](../../docs/compliance-read-model-design.md) — detailed implementation plan
