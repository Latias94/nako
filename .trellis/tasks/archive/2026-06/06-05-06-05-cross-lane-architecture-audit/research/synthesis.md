# Cross-Lane Architecture Audit Synthesis

## Inputs

- `research/playback-decoding-transcode.md`
- `research/remote-access-network-operations.md`
- `research/addon-boundary-automation.md`
- `research/admin-api-web-contracts.md`
- `research/storage-library-control-plane.md`

## Executive Decision

Use a **mixed plan**:

1. Run a small Admin/API contract-hardening task first or as the contract-owner
   lane.
2. Run architecture audits for areas where the next implementation would set
   durable contracts.
3. Run low-conflict ready implementation/docs tasks in parallel.
4. Defer broad fearless refactor cleanup until a chosen lane creates concrete
   pressure in the files to be cleaned.

The audit does **not** support starting a global fearless refactor campaign now.
Most reviewed lanes already have meaningful modules and interfaces. The
remaining pressure is contract choice, route/generator drift prevention, and
product boundary decisions.

## Cross-Lane Findings

### 1. Admin/API Is The Shared Serialization Surface

`nako-api`, Admin route inventory, generated TypeScript contracts, Admin Web
route state, and redaction tests are the most likely conflict point for parallel
work. Playback diagnostics, remote access readiness, Addon Manager, VFS cache
repair, source hash jobs, and durable job surfaces all eventually touch this
area.

Highest-value immediate task:

- `admin-route-inventory-parity-gate`

Why:

- It protects every future Admin feature lane.
- It is bounded and implementation-ready.
- It prevents generated route constants, Axum route registration, and Admin Web
  client helpers from drifting while multiple lanes run.

### 2. Playback/Decoding Needs Output Capability Architecture Before More Execution

Playback/transcode is not shallow. `nako-playback`, `nako-transcode`, and
server playback flow modules mostly honor ADR 0038/0044/0045/0049/0052/0053.

The next bottleneck is not refactor; it is deciding the capability contract for
Client Applications and output profiles:

- HEVC/AV1 output,
- hardware tone mapping,
- image subtitle burn-in,
- richer device profiles,
- mobile/TV/native client behavior.

Highest-value task:

- `playback-output-profile-and-device-capability-audit`

### 3. Remote Access Has A Ready Low-Conflict Ops Slice

Remote access policy/readiness is already shipped. Nako should not jump into
network tunnel runtime or endpoint discovery implementation yet.

Ready tasks:

- `self-hosted-remote-access-cookbook`
- `remote-access-config-fixture-release-gate`

Architecture tasks:

- `client-endpoint-selection-architecture`
- `network-tunnel-provider-runtime-decision`

Fearless refactor candidate:

- `network-policy-classifier-deepening`, but only when adding new network
  readiness states.

### 4. Addon Boundary Is Strong Enough, But Host-Owned Flows Need A Pattern

Addon Protocol, Addon Sidecar, grants/tokens, event delivery, task runs, and
Generated Artifact boundaries are not missing. The main risk is repeated
host-owned resource-flow logic across Resource Search, subtitles, and external
acquisition.

Highest-value architecture task:

- `host-owned-addon-resource-flow-pattern`

Second architecture task:

- `addon-task-event-execution-policy-convergence`

Product PRD:

- `addon-manager-first-product-slice`

Do not start by mechanically splitting `nako-addon-protocol`, client, or
official catalog large files. Size alone was not enough evidence for a valuable
refactor.

### 5. Storage/VFS Source Hash Is Mature; Triggering And Reconciliation Need Policy

Source fingerprint hash has shipped its first queue/execution/evidence/Admin
visibility path. The remaining work is not "prove it can execute"; it is
deciding:

- how work is triggered beyond the internal app service,
- whether Admin/API manual triggering is in scope,
- whether persisted hash evidence creates Source Duplicate Relationship
  suggestions,
- how automatic reconciliation is bounded and reversible.

Highest-value architecture task:

- `source-fingerprint-hash-triggering-and-reconciliation-policy`

Ready bounded tasks:

- `vfs-cache-non-destructive-remediation-plan`
- `storage-source-identity-postgres-runtime-harness`

## Recommended Parallel Queue

### Lane A: Admin Contract Gate

Task: `admin-route-inventory-parity-gate`

Type: ready bounded implementation.

Why first:

- It reduces cross-lane merge risk for every Admin-facing follow-on.
- It can become the contract-owner gate while other lanes remain docs/audit
  only.

Conflict rules:

- Owns `nako-api/src/admin_contract.rs`, generated Admin TypeScript contracts,
  and route inventory tests.
- No other lane should regenerate Admin contracts while this runs.

### Lane B: Playback Capability Audit

Task: `playback-output-profile-and-device-capability-audit`

Type: architecture audit.

Why parallel-safe:

- Can stay docs/research first.
- Should not modify Public Client playback DTOs or `nako-playback` structs
  during the audit.

Output:

- chosen contract shape for output codec/container/subtitle/HDR/device facts;
- split of Public Client facts versus Admin diagnostics;
- first executable target recommendation for HEVC/AV1, hardware tone mapping,
  or subtitles.

### Lane C: Remote Access Ops Slice

Task bundle:

- `self-hosted-remote-access-cookbook`
- `remote-access-config-fixture-release-gate`

Type: ready docs/ops implementation.

Why parallel-safe:

- Mostly touches deployment docs, examples, release gates, and config-check
  fixtures.
- Does not require Public Client endpoint discovery or tunnel runtime.

### Lane D: Addon Boundary Audit

Task: `host-owned-addon-resource-flow-pattern`

Type: architecture audit before implementation.

Why parallel-safe:

- Can remain design/spec first.
- Should avoid Admin DTO changes until Lane A finishes.

Output:

- common server-side pattern for selection sessions, apply plans, selected
  references, safe error codes, grant checks, redaction, and side-effect
  handoff.

### Lane E: Storage/Control-Plane Policy Audit

Task: `source-fingerprint-hash-triggering-and-reconciliation-policy`

Type: architecture audit before implementation.

Why parallel-safe:

- Can stay docs/research first.
- Must not edit `jobs.rs`, source identity repository contracts, or Admin DTOs
  while Lane A or durable-job migration work is active.

Output:

- policy for scan-originated enqueue, Admin manual enqueue, retry/requeue, and
  automatic source duplicate suggestion behavior.

## Secondary Ready Tasks

These are good follow-ons after the initial queue starts:

| Candidate | Lane | Type | Notes |
| --- | --- | --- | --- |
| `admin-playback-support-source-facts-first-slice` | playback/admin | implementation | Only after Admin contract owner clears DTO scope. |
| `vfs-cache-non-destructive-remediation-plan` | storage-vfs | implementation | Avoid concurrent storage DTO changes. |
| `storage-source-identity-postgres-runtime-harness` | storage/db | verification | Low product ambiguity; good confidence task. |
| `admin-web-route-search-helper` | admin-web | fearless refactor | Useful when page churn resumes. |
| `separate-admin-client-from-public-client-bridges` | admin-web/client | fearless refactor | Useful if web-product and client-surface work run together. |
| `remote-admin-network-diagnostics-drilldown` | remote/admin | implementation | Only if Admin DTO scope is free. |

## Refactor Readiness

### Ready Or Nearly Ready

- Admin Web route search helper.
- Separate Admin client from Public Client bridges.
- Admin contract generator deepening, but only after route inventory gate and a
  short design pass.

### Defer

- Playback artifact identity and flow cleanup: valid but should piggyback on a
  chosen playback implementation target.
- Network policy classifier: valuable when new network readiness states are
  added, not before docs-only cookbook work.
- Addon host-owned resource flow extraction: needs one design task first; do
  not push policy into `nako-addon-protocol`.
- Disk-scan job executor registry for source hash/library scan: only two job
  variants exist; a generic registry would be hypothetical today.

## Unsafe Parallel Combinations

- Two tasks regenerating Admin TypeScript contracts.
- Endpoint discovery implementation with client SDK/playback/cast transport
  work.
- HEVC/AV1 output execution with hardware tone-map execution.
- Source hash triggering with broad durable job scheduler migration.
- Addon Manager UI with Admin Addon route/generated contract changes unless
  one lane owns the contract.
- Trusted proxy/header behavior with auth or trace middleware changes.
- VFS cache Admin DTO changes with any other storage/Admin diagnostics DTO
  task.

## Documentation Updates Needed

- `docs/architecture/LANES.md` should stop naming already-shipped source hash
  Admin diagnostics as a future action.
- `docs/architecture/STORAGE_VFS.md` should use archived task paths for shipped
  VFS cache repair tasks and distinguish refresh-only actions from durable
  repair queues.
- `docs/architecture/CONTROL_PLANE.md` should distinguish shipped remote access
  readiness from unstarted endpoint discovery, and shipped source hash Admin
  reads from pending Admin/API triggering.
- `docs/architecture/PLAYBACK.md` should reconcile shipped seek query/request
  identity work from remaining seek/keyframe/player validation work.
- `docs/architecture/OPERATIONS_RELEASE.md` should name remote access cookbook
  and config-check fixtures as ready ops follow-ons.

## Final Recommendation

Do **not** choose between "architecture audit" and "fearless refactor" globally.
Use a lane-specific mix:

- First: Admin route inventory parity gate.
- In parallel: playback output/device capability audit, remote access cookbook
  and config-check fixtures, Addon host-owned resource flow audit, and source
  fingerprint hash triggering/reconciliation audit.
- Then: choose one or two implementation follow-ons and one cleanup lane based
  on the audit outputs.

This gives parallel agents stable contracts before they start editing the same
Admin/API/runtime surfaces.
