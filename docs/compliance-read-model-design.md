# Compliance Read Model — Technical Design

> Companion document to [ADR-005: Unify SCA Compliance Views](architecture/adr/adr-005-unify-sca-compliance-views.md).

## Overview

This document describes the technical implementation for enriching SCA results with corporate compliance profiles and making them available in the Wazuh Dashboard via a dedicated OpenSearch index.

> For architectural context, decision rationale, and trade-offs, see [ADR-005](architecture/adr/adr-005-unify-sca-compliance-views.md).

---

## Design Goals

The implementation must:

- Provide a centralized compliance read model for SOC analysts
- Preserve the existing compliance evaluation logic in the gateway
- Avoid duplicating compliance rules in dashboard tooling
- Support both fleet-level visibility and check-level investigation
- Allow future consumers without changing the compliance engine
- Remain operationally simple by reusing existing infrastructure

---

## Data Model

### Data Ownership

| Category         | Owner                          | Examples                                                                               |
| :--------------- | :----------------------------- | :------------------------------------------------------------------------------------- |
| **Source data**  | Wazuh SCA + Corporate profiles | Raw check results, profile definitions                                                 |
| **Domain model** | Gateway Compliance Engine      | Compliance score, compliance_status, category mappings, remediation, summary documents |
| **Projection**   | OpenSearch Read Model          | Dashboard queries, fleet-level aggregation                                             |

> OpenSearch data is read-only and cannot be manually modified. Derived data is overwritten on each sync cycle.

### Check-Level Document

Each document represents one check for one agent. Used for drill-down views.

```json
{
  "type": "check",
  "agent_id": "001",
  "agent_name": "web-server-01",
  "os": "Ubuntu",
  "policy_id": "cis_ubuntu20-04",
  "policy_name": "CIS Ubuntu Linux 24.04 LTS Benchmark v1.0.0",
  "scan_id": "1023532995",
  "profile_version": "1.2.0",
  "@timestamp": "2026-08-21T10:30:00Z",

  "category": "Firewall",
  "check_id": 28573,
  "check_title": "Ensure ufw is installed",
  "check_status": "failed",
  "mandatory": true,
  "remediation": "Install UFW: 'sudo apt install ufw'...",

  "compliance_standards": {
    "cis": ["4.2.1"]
  }
}
```

> **Note:** Report-level fields (`score`, `compliance_status`) belong to the summary document only. Check-level documents contain check-specific data to avoid duplication and consistency issues.

### Summary Document

One summary document per agent provides the aggregated view for fleet-level queries.

```json
{
  "type": "summary",
  "agent_id": "001",
  "agent_name": "web-server-01",
  "os": "Ubuntu",
  "profile_version": "1.2.0",
  "score": 85,
  "compliance_status": "non-compliant",
  "total_passed": 12,
  "total_failed": 2,
  "total_untested": 3,
  "categories": [
    { "name": "Firewall", "passed": 3, "failed": 1, "status": "non-compliant" },
    { "name": "SSH Hardening", "passed": 4, "failed": 0, "status": "compliant" }
  ],
  "@timestamp": "2026-08-21T10:30:00Z"
}
```

### Why Both Document Types

We maintain both granular and aggregated documents because SOC analysts require fast fleet-level queries (e.g., "show me all non-compliant agents") while still requiring drill-down capability (e.g., "which specific checks are failing on agent X?"). This follows a CQRS-inspired approach by maintaining a pre-computed read model optimized for dashboard queries: aggregated summaries for fleet-level views, detailed documents for investigation.

### compliance_status Semantics

The gateway determines `compliance_status` at the **report level**:

- `"compliant"` — all mandatory checks pass
- `"non-compliant"` — any mandatory check fails

Category-level `status` is derived from whether any check in that category failed, but it does not affect the overall `compliance_status`. A failed optional check does not make an agent non-compliant.

### Document IDs

Documents use deterministic IDs for idempotent re-indexing:

- Check-level: `{agent_id}-{policy_id}-check-{check_id}`
- Summary: `{agent_id}-summary`

Including `policy_id` in the check-level ID prevents collisions when the same `check_id` exists across different policies. Document IDs intentionally exclude `profile_version` because profile changes should update the existing compliance posture instead of creating duplicate historical documents.

### Score Calculation

The compliance score is calculated as:

```
score = round((passed / (passed + failed)) × 100)
```

**Untested checks are excluded** from the score calculation because they do not represent evaluated compliance failures. A check is untested when the agent's SCA scan did not produce a result for that check (e.g., a service is inactive on the machine).

Example: `passed=80, failed=10, untested=10` → `score = 80/(80+10) = 89`

### Profile Versioning

Each document includes a `profile_version` field indicating which version of the corporate compliance profile was used for evaluation. This enables:

- **Audit trail** — Determining which rules were active when a compliance result was computed
- **Historical debugging** — Understanding why an agent's status changed (profile update vs SCA scan change)
- **Rollback safety** — Reverting a profile change and re-syncing produces results traceable to the new version

Profile version is derived from the compliance profile files (e.g., git tag or semantic version in the profile metadata).

---

## Compliance Evaluation Flow

This is the core transformation performed by the gateway on each sync cycle:

```
  Wazuh Check Result
        │
        v
  Corporate Profile Matcher
  (match by title, compliance standard, or check ID)
        │
        v
  Compliance Evaluator
  (apply mandatory rules, compute score)
        │
        v
  Compliance Report
        │
        +──► OpenSearch Read Model
        │
        +──► Gateway API Response
```

**Steps:**

1. Fetch SCA policies for the agent from Wazuh Manager
2. Retrieve the corporate compliance profile matching the agent's OS
3. Match corporate checks against Wazuh SCA check results (by title, compliance standard overlap, or check ID)
4. Apply mandatory rules — determine overall `compliance_status` at report level
5. Calculate compliance score: `passed / (passed + failed) × 100`
6. Generate `ComplianceReport` with category breakdowns and per-check statuses
7. Persist to OpenSearch read model (check-level + summary documents)

> This evaluation logic already exists in `wazuh-gateway/src/compliance/mod.rs`. No new evaluation code is needed — only the OpenSearch persistence layer is added.

---

## Sync Architecture

### Data Flow

```
                 Corporate Profiles
                       │
                       │
Wazuh Manager ────► Compliance Engine ────► OpenSearch Index
  (raw SCA)        (wazuh-gateway)         (read model)
                       │                        │
                       │ enrichment             │
                       │                        ▼
                       │                   Dashboard
                       │
                       ▼
                  Compliance API
                  (desktop client)
```

The gateway is not just moving data — it **transforms** raw SCA results into an enriched compliance model by applying corporate profiles, computing scores, and assigning categories. The gateway calls the Wazuh Manager API directly via HTTP (`/sca/*` endpoints) — it does not read SCA data from OpenSearch.

### Detailed Sync Flow

```
  ┌──────────────┐                  ┌──────────────────┐
  │  Wazuh       │    Read query    │  CronJob (K8s)   │
  │  Dashboard   │ ◄─────────────── │  Every N hours   │
  │              │                  └────────┬─────────┘
  │  + Custom    │     OpenSearch             │
  │  Compliance  │     ┌──────────────────────┼──── POST /compliance/sync
  │  Dashboard   │     │                      │
  │              │     │  wazuh-custom-       │
  │  [Refresh    │ ◄───│  compliance-*        │
  │   Button]────│─POST│                      │
  │  (on-demand) │     └──────────────────────┘
  └──────────────┘               │
                                 │ bulk index
                                 │
              ┌──────────────────┴───────────────┐
              │  wazuh-gateway (extended)         │
              │                                   │
              │  sync_all_agents():               │
              │  1. GET /agents (active)          │
              │  2. Concurrent fetch              │
              │  3. Incremental (skip unchanged)  │
              │  4. evaluate_compliance()         │
              │  5. Bulk index to OpenSearch      │
              │  6. Backoff on rate limits        │
              │                                   │
              └──────────────────┬───────────────┘
                                 │
                                 ▼
              ┌──────────────────────────────────┐
              │  Wazuh Manager API               │
              │  /agents, /sca/*                 │
              └──────────────────────────────────┘
```

### Sync Control

Both mechanisms use the same `POST /compliance/sync` endpoint and identical indexing logic.

**1. Automatic (Scheduled)** — A Kubernetes CronJob calls `POST /compliance/sync` every N hours (configurable, default: 4 hours). Keeps the dashboard data fresh without manual intervention.

**2. On-Demand (SOC Analyst)** — The dashboard includes a **"Refresh Compliance"** button that initiates an asynchronous re-sync via the gateway API. The operation is protected by the same synchronization lock used by scheduled jobs — multiple simultaneous refresh requests reuse the existing synchronization task. This prevents Wazuh API overload from concurrent user requests.

The dashboard does not execute synchronization directly on behalf of the user session. It sends an authenticated command request to the gateway, which controls execution. The dashboard provides query capability through the read model and command initiation through authenticated gateway API calls.

### Gateway API Contracts

#### Trigger compliance synchronization

```
POST /compliance/sync
```

Used by both the CronJob and the dashboard refresh button. Returns immediately so the caller is not blocked during large fleet syncs.

**Response:**

```json
{
  "status": "accepted",
  "sync_id": "abc123"
}
```

**Response (sync already in progress):**

```json
{
  "status": "already_running",
  "sync_id": "existing-id-456"
}
```

#### Check synchronization status

```
GET /compliance/sync-status/{sync_id}
```

**Response (in progress):**

```json
{
  "sync_id": "abc123",
  "status": "running",
  "agents_completed": 45,
  "agents_total": 120,
  "started_at": "2026-08-21T10:30:00Z"
}
```

**Response (completed):**

```json
{
  "sync_id": "abc123",
  "status": "completed",
  "agents_completed": 120,
  "agents_total": 120,
  "agents_synced": 115,
  "agents_skipped": 5,
  "agents_failed": 0,
  "started_at": "2026-08-21T10:30:00Z",
  "completed_at": "2026-08-21T10:31:45Z"
}
```

### Sync Job Coordination

Only one compliance synchronization job may run at a time. If another sync is requested while one is already running:

- Return the existing `sync_id` with status `already_running`
- The new caller polls `GET /compliance/sync-status/{sync_id}` for completion

This prevents duplicate Wazuh API calls, OpenSearch write conflicts, and unnecessary load.

### Sync State Management

The gateway stores synchronization metadata to support incremental sync:

```json
{
  "agent_id": "001",
  "last_processed_scan_id": "1023532995",
  "last_processed_profile_version": "1.2.0",
  "last_sync_timestamp": "2026-08-21T10:30:00Z"
}
```

**Storage:** OpenSearch metadata index (`wazuh-custom-compliance-meta`). This survives pod restarts and avoids local filesystem dependency in Kubernetes.

Persistent synchronization metadata (agent scan IDs, profile versions, last sync timestamps) survives restarts through the OpenSearch metadata index. Runtime execution state (active jobs, progress tracking) is held in memory and recreated after restart — which means a restart triggers a full re-sync (safe default).

An agent is skipped only when **both** conditions are met:

- SCA `scan_id` has not changed
- Compliance `profile_version` has not changed

A profile update (e.g., adding a mandatory flag, renaming a category) invalidates previous evaluations and triggers reprocessing of all affected agents.

---

## Failure Handling

> Architectural risks and mitigations are documented in the ADR. This section covers implementation-level failure behavior.

| Scenario                      | Behavior                                                                          |
| :---------------------------- | :-------------------------------------------------------------------------------- |
| Single agent API failure      | Agent is skipped, logged in sync status, retried next cycle                       |
| OpenSearch bulk index failure | Retries with exponential backoff; existing data remains queryable                 |
| Gateway crash during sync     | In-memory sync state is lost; next cycle re-syncs all agents (safe default)       |
| Partial OpenSearch write      | Deterministic doc IDs ensure idempotency; partial writes are overwritten on retry |

---

## Security Considerations

- Dashboard access follows existing Wazuh authentication and authorization
- Compliance data in OpenSearch is read-only from the dashboard's perspective
- Sync endpoints (`POST /compliance/sync`) are restricted to internal services and authenticated admin users
- OpenSearch credentials are managed through Kubernetes secrets
- The gateway authenticates to the Wazuh Manager API using existing API credentials

---

## Performance and Scalability

### Concurrent Agent Processing

Agents are processed in parallel using Tokio tasks with a configurable concurrency limit (default: 10). This keeps Wazuh API pressure bounded while maximizing throughput.

```
~2 API calls per agent (policies + checks)
~2 seconds per agent (API latency)
With 10 concurrent: agents complete in ~2s each × (N ÷ 10) batches
```

### Incremental Sync

Each agent's SCA `scan_id` is stored. On the next sync, only agents whose `scan_id` changed since the last sync are re-evaluated. Most agents will not have changed between sync cycles, so the typical sync processes only a fraction of the fleet.

### Batched OpenSearch Indexing

All documents for an agent are indexed in a single `_bulk` call with deterministic IDs. Re-indexing the same agent overwrites previous data idempotently.

### Rate Limit Awareness

The Wazuh Manager API has rate limits. The sync endpoint respects HTTP 429 responses with exponential backoff, ensuring it does not overwhelm the Manager during fleet syncs.

---

## OpenSearch Index Mapping

The compliance index should define explicit mappings instead of relying on dynamic mappings. This prevents mapping conflicts and ensures predictable dashboard behavior.

| Field                  | Type      | Notes                               |
| :--------------------- | :-------- | :---------------------------------- |
| `agent_id`             | `keyword` | Exact match, used for filtering     |
| `agent_name`           | `keyword` | Exact match                         |
| `os`                   | `keyword` | Exact match                         |
| `policy_id`            | `keyword` | Exact match                         |
| `policy_name`          | `text`    | Full-text searchable                |
| `scan_id`              | `keyword` | Exact match                         |
| `profile_version`      | `keyword` | Exact match                         |
| `@timestamp`           | `date`    | Used for time-range queries         |
| `score`                | `integer` | 0–100 range                         |
| `compliance_status`    | `keyword` | `compliant` or `non-compliant`      |
| `category`             | `keyword` | Exact match, used for aggregation   |
| `check_id`             | `integer` | Exact match                         |
| `check_title`          | `text`    | Full-text searchable                |
| `check_status`         | `keyword` | `passed`, `failed`, or `untested`   |
| `mandatory`            | `boolean` | Filter for mandatory checks         |
| `remediation`          | `text`    | Full-text searchable                |
| `compliance_standards` | `nested`  | Nested compliance standard mappings |

---

## OpenSearch Dashboard

### Index Pattern

`wazuh-custom-compliance-*`

### Dashboard Panels

| Panel                      | Description                                                                                    |
| :------------------------- | :--------------------------------------------------------------------------------------------- |
| **Fleet Compliance Score** | Gauge chart showing fleet-wide average compliance score                                        |
| **Agent Compliance Table** | Table of all agents with score, compliance_status, passed/failed counts — sortable, filterable |
| **Non-Compliant Agents**   | Filtered view showing agents with `compliance_status: non-compliant`                           |
| **Category Breakdown**     | Bar chart showing pass/fail distribution across categories                                     |
| **Mandatory Failures**     | Table filtered to `mandatory: true` and `check_status: failed`                                 |
| **Check Drill-Down**       | Click an agent to see all individual checks with status and remediation                        |
| **Manual Refresh Button**  | Triggers on-demand compliance re-sync via gateway API                                          |

---

## Implementation Tasks

### Phase 1: Gateway Extension

| Step | Description                                                                                             | Files                                |
| :--- | :------------------------------------------------------------------------------------------------------ | :----------------------------------- |
| 1.1  | Add OpenSearch client dependency (`opensearch` crate)                                                   | `Cargo.toml`                         |
| 1.2  | Add OpenSearch connection config                                                                        | `src/config.rs`                      |
| 1.3  | Implement `bulk_index_compliance()` — indexes ComplianceReport to OpenSearch with deterministic doc IDs | `src/compliance/mod.rs`              |
| 1.4  | Implement compliance synchronization workflow — concurrent agent processing, incremental sync           | `src/handlers/mod.rs`                |
| 1.5  | Add `POST /compliance/sync` and `GET /compliance/sync-status/{sync_id}` endpoints                       | `src/handlers/mod.rs`                |
| 1.6  | Add rate limit handling — exponential backoff on HTTP 429                                               | `src/client/mod.rs`                  |
| 1.7  | Add Kubernetes CronJob manifest                                                                         | `wazuh-helm/charts/wazuh/templates/` |

### Phase 2: OpenSearch Dashboards

| Step | Description                                      |
| :--- | :----------------------------------------------- |
| 2.1  | Create index pattern `wazuh-custom-compliance-*` |
| 2.2  | Build dashboard saved objects (JSON export)      |
| 2.3  | Import via OpenSearch Dashboards API or UI       |

### Phase 3: Validation

| Step | Description                                                     |
| :--- | :-------------------------------------------------------------- |
| 3.1  | Verify scores match between desktop client and Wazuh Dashboard  |
| 3.2  | Test filtering by agent, compliance_status, mandatory, category |
| 3.3  | Verify indexing works across Linux, macOS, and Windows agents   |
| 3.4  | Load test with realistic fleet size                             |

---

## Open Questions

- What operational thresholds should trigger extracting synchronization into a dedicated worker (e.g., sync duration, Wazuh API utilization, gateway latency impact)?
- Should compliance history be retained, or only the latest state per agent?
- Should OpenSearch store only current posture, or historical snapshots for trend analysis?
- What is the expected maximum agent fleet size for capacity planning?
- Should the refresh button show a progress indicator during sync, or just a notification on completion?

---

## References

- [ADR-005: Unify SCA Compliance Views](architecture/adr/adr-005-unify-sca-compliance-views.md)
- [wazuh-gateway SCA Doc](../wazuh-gateway/doc/sca.md)
- [Compliance Profiles](https://github.com/ADORSYS-GIS/wazuh-gateway/tree/main/src/compliance_profiles)
