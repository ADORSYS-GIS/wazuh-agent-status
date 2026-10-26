# ADR-005: Query Local Endpoint SCA SQLite Database in wazuh-agent-status while Retaining Gateway for v4

> We will update wazuh-agent-status to read Security Configuration Assessment (SCA) results directly from the local agent SQLite database (`queue/sca/db/sca.db` across Linux, macOS, and Windows) on endpoints, while retaining the wazuh-gateway Helm deployment in the cluster to support remaining Wazuh 4.x users until fleet migration is complete.

## Metadata

| Field | Value |
| :--- | :--- |
| **Document ID** | ADR-005 |
| **Status** | `Accepted` |
| **Date** | 2026-10-26 |
| **Recorded** | 2026-10-26 |
| **Deciders** | SIEM team |
| **Governs** | `charts/wazuh-gateway`, `wazuh-gateway`, `wazuh-agent-status-rust-client`, `wazuh-agent-status-rust-server`, `queue/sca/db/sca.db` |
| **Amends** | N/A |
| **Amended by** | N/A |
| **Supersedes** | N/A |
| **Superseded by** | N/A |

## Context

In Wazuh 4.x, employee endpoints running `wazuh-agent-status` retrieved Security Configuration Assessment (SCA) compliance results by contacting `wazuh-gateway` over HTTPS on port 8080 with HMAC-SHA256 headers (`x-agent-proof`, `x-agent-timestamp`, `x-agent-nonce`). `wazuh-gateway` authenticated the caller against the agent key from the Wazuh Manager, fetched raw check results via the Manager REST API at `GET /sca/{agent_id}` and `GET /sca/{agent_id}/checks/{policy_id}`, evaluated the checks against corporate JSON profiles (`linux.json`, `macos.json`, `windows.json`), and returned the compliance score.

With the upgrade to **Wazuh v5.0**, the Wazuh Manager API deleted the `/sca` REST endpoints entirely. Issuing `GET /sca/{agent_id}` against the Manager API returns `HTTP 404 Not Found: {"title": "Not Found", "detail": "404: Not Found"}`. Consequently, `wazuh-gateway` fails with `AppError::WazuhApiError`, returning `500/502` to desktop clients and failing scheduled compliance sync runs (`synced=0, failed=N`).

Simultaneously, the Wazuh v5.0 agent persists all local SCA check evaluations on the host endpoint inside a local SQLite database at `queue/sca/db/sca.db` (located at `/var/ossec/queue/sca/db/sca.db` on Linux, `/Library/Ossec/queue/sca/db/sca.db` on macOS, and `C:\Program Files (x86)\ossec-agent\queue\sca\db\sca.db` on Windows).

However, the organization still has active endpoints running **Wazuh Agent version 4.x** that depend on `wazuh-gateway`. Immediately removing the gateway Helm release would disrupt compliance reporting for these legacy users.

The forces deciding this record were maintaining uninterrupted service for legacy v4 endpoints, eliminating network/VPN dependencies for v5 endpoints, and avoiding complex multi-version branching inside `wazuh-gateway`.

## Decision

We will:
1. Retain the `wazuh-gateway` Helm deployment (`charts/wazuh-gateway`) and LoadBalancer service running in the Kubernetes cluster in maintenance mode to continue serving legacy Wazuh 4.x endpoints.
2. Update `wazuh-agent-status` to query the local agent SQLite database (`queue/sca/db/sca.db`) directly on the local machine using read-only SQLite connections (`PRAGMA query_only = ON`) for all Wazuh v5 endpoints across Linux, macOS, and Windows.
3. Retrieve all SCA check records from `sca_check` (specifically `id`, `name`, `result`, `rationale`, and `remediation`) without corporate policy filtering for the initial release.
4. Access the database file via `wazuh-agent-status-rust-server` (running as a system daemon/service with local filesystem privileges) or via group/file permissions on `queue/sca/db/`, passing the structured check list over local IPC to the Tauri frontend.
5. Defer the permanent decommissioning of `wazuh-gateway` in Helm until the SIEM team confirms 100% of employee endpoints have upgraded to Wazuh v5.0.

## Options considered

| Option | Verdict | Why (one line) |
| :--- | :--- | :--- |
| Retain gateway for v4 users; query local endpoint SQLite (`queue/sca/db/sca.db`) in `wazuh-agent-status` for v5 | Adopted | Solves the v5 `/sca` 404 breakdown with 100% offline capability without stranding legacy v4 endpoints. |
| Immediately remove `wazuh-gateway` Helm deployment | Rejected | Breaks compliance reporting for existing company endpoints still running Wazuh 4.x. |
| Refactor `wazuh-gateway` to query OpenSearch (`wazuh-states-sca`) for v5 | Rejected | Retains network and VPN dependency for endpoints, requires HMAC signing maintenance, and adds read query load to OpenSearch. |
| Direct desktop client queries to OpenSearch (`https://<indexer>:9200`) | Rejected | Requires distributing OpenSearch credentials to all corporate laptops, exposing indexer ports externally, and enforcing complex Document-Level Security. |

## Consequences

**Not part of this decision:** Corporate profile filtering and custom compliance scoring (e.g. evaluating specific CIS rules against corporate baselines). For now, `wazuh-agent-status` displays all checks returned by the local database; custom category filtering and weighting will be specified in a follow-up architectural decision. Central aggregation of compliance posture into OpenSearch (`wazuh-custom-compliance-*`) is also deferred.

### Impact

**Benefits**
- Existing Wazuh 4.x endpoints experience zero disruption and continue receiving compliance scores via `wazuh-gateway`.
- Upgraded Wazuh v5 endpoints gain instant, 100% offline compliance status without needing VPN or network connectivity.
- Zero network latency for v5 compliance evaluations.
- Decouples endpoint desktop app development from central cluster release schedules.

**Costs**
- `wazuh-gateway` remains deployed in the Kubernetes cluster consuming resources until v4 endpoints are phased out.
- `wazuh-agent-status` must include an embedded SQLite reader (`rusqlite`) and handle local filesystem permissions across operating systems.

**Risks**
- Running a dual-path architecture means legacy v4 users see filtered corporate reports via gateway, while v5 users initially see unfiltered raw check lists until filtering parity is implemented.
- Upstream changes to the internal table schema of `sca.db` in future Wazuh minor releases could break the local query parser (blast radius limited to the desktop client SCA view).

### Accepted limits

- **Dual operational paths:** The infrastructure team must maintain `charts/wazuh-gateway` in Kubernetes until all endpoints upgrade to v5.
- **Unfiltered check results for v5:** Endpoints display the complete raw list of CIS benchmark checks from `sca_check` without corporate policy category filtering until the filtering mechanism is added.
- **Local filesystem access:** Non-root desktop applications cannot read the agent queue directory directly without the `wazuh-agent-status-rust-server` daemon/service running with appropriate read privileges.

### Done when

- [ ] `wazuh-agent-status-rust-server` reads `queue/sca/db/sca.db` and extracts check records from `sca_check` across Linux, macOS, and Windows.
- [ ] `wazuh-agent-status-rust-client` displays the local scan results with Pass/Fail/Not applicable statuses on v5 endpoints without contacting `wazuh-gateway:8080`.
- [ ] `wazuh-gateway` continues operating in `wazuh` namespace servicing incoming v4 HMAC requests.
- [ ] Compliance status successfully displays while a v5 endpoint is disconnected from network and VPN.

### Revisit when

- All legacy employee endpoints are upgraded to Wazuh v5.0, triggering the final decommissioning and removal of `charts/wazuh-gateway`.
- Wazuh modifies the internal table schema of `sca.db` in a future agent release.

## Rollback

**When.** Local SQLite queries consistently fail across Windows/macOS/Linux on v5 endpoints due to unresolvable operating system permission constraints, or read contention corrupts/locks the agent database during active scans.

**How, in order.**

1. Revert the commit in `wazuh-agent-status` that points `fetch_compliance` to the local SQLite parser, returning to HTTP client calls.
2. In `wazuh-gateway`, implement the OpenSearch proxy endpoint (`wazuh-states-sca`) to service v5 agents over HTTP port 8080.
3. Verify `GET /health` and HMAC signature validation on `http://<gateway>:8080/agents/{id}/compliance` for both v4 and v5 agents.
4. Remove local file reading permissions and helper configurations from endpoint installations.

## Evidence

| Claim | Verification Command / Source | Expected / Observed Outcome |
| :--- | :--- | :--- |
| Manager REST API `/sca` endpoints removed in v5.0 returning 404 | `curl -sk -i -H "Authorization: Bearer $TOKEN" https://<manager-host>:55000/sca/<agent_id>` | Returns `HTTP/1.1 404 Not Found: {"title": "Not Found", "detail": "404: Not Found"}` |
| Local agent SQLite database exists and contains evaluated check records | `sqlite3 <agent-path>/queue/sca/db/sca.db "SELECT result, count(*) FROM sca_check GROUP BY result;"` | Returns counts of evaluated checks grouped by result (e.g. `Passed`, `Failed`, `Not applicable`) |
| Gateway compliance sync fails active agents in Wazuh v5.0 clusters | Pod logs: `kubectl -n <namespace> logs deployment/wazuh-gateway` | Output confirms: `Compliance synchronization completed synced=0 skipped=0 failed=N` |
| Raw check JSON schema in local SQLite matches requirements | `sqlite3 -json <agent-path>/queue/sca/db/sca.db "SELECT id, name, result, rationale, remediation FROM sca_check LIMIT 1;"` | Returns valid JSON object with check attributes matching desktop application schema |

## References

- [Wazuh Issue #29533: States persistence](https://github.com/wazuh/wazuh/issues/29533)
- [Wazuh Issue #23393: SPIKE - Initial registration system design](https://github.com/wazuh/wazuh/issues/23393)
- [Wazuh Release v5.0.0-rc1](https://github.com/wazuh/wazuh/releases/tag/v5.0.0-rc1)

## Solution implementation

| Kind | Tracking | Status | What it covers |
| :--- | :--- | :--- | :--- |
| Issue | | Proposed | Implement SQLite parser for `queue/sca/db/sca.db` across platforms |
| Task | | Proposed | Maintain gateway in Helm for v4 backwards compatibility |
| Task | | Deferred | Decommission `charts/wazuh-gateway` once v4 phase-out is complete |
| PR | | Proposed | Implementation pull requests across repositories |
