# Source Fingerprint Hash Triggering And Reconciliation Policy Audit

## Scope

This audit decides how source fingerprint hash work should be triggered after
the shipped internal queue/executor path, and whether persisted hash evidence
should mutate Source Duplicate Relationship records in the next slice.

This is audit-only. It does not add API routes, job enqueue behavior, schema
migrations, duplicate relationship writes, or a source-hash-specific runtime
loop.

Primary evidence:

- `CONTEXT.md`
- `docs/adr/0012-durable-scan-state-and-source-tombstones.md`
- `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`
- `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
- `docs/adr/0053-application-control-plane-boundary.md`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/CONTROL_PLANE.md`
- `crates/nako-core/src/media/source.rs`
- `crates/nako-library/src/ingestion/source_commit.rs`
- `crates/nako-library/src/source_hash.rs`
- `crates/nako-server/src/app/source_hash.rs`
- `crates/nako-server/src/app/jobs.rs`
- `crates/nako-api/src/admin/operations.rs`
- `crates/nako-api/src/admin.rs`
- `crates/nako-db/src/sqlite/source_duplicate.rs`
- `crates/nako-db/src/postgres/core_catalog.rs`

## Current State

### Source Fingerprint Policy

`nako-core::SourceFingerprintEvidence` already distinguishes evidence kind,
confidence, stale state, and escalation decisions. A fingerprint can preserve
source identity only when evidence is non-stale, has confidence >= 900, and has
fingerprint material. It can suggest duplicates at confidence >= 500.

This matches the domain rule:

- Source Fingerprint is evidence.
- Source Locator remains the library-scoped address.
- Source Duplicate Relationship is separate from merged source identity.

### Scan Source Commit

`nako-library::ingestion::source_commit` computes a
`SourceFingerprintEscalationDecision` during source observation planning. It
does not hash bytes and does not enqueue jobs.

Current scan commit behavior:

- Existing locator: update the existing Media Source.
- Strong non-stale evidence with one eligible old source: preserve source
  identity as a relocation/update.
- Weak or ambiguous duplicate candidates: create Suggested Source Duplicate
  Relationship records in the scan commit.
- Partial/full hash recommendations are advisory fields on the in-memory
  source observation plan.

The scan path remains the correct place to observe evidence and recommend
hashing, not to run VFS reads or schedule hidden work inside the pure commit
planner.

### Hash Scheduling, Queue, Execution, And Evidence Persistence

`nako-library::source_hash` already has:

- `SourceFingerprintHashMode`
- `SourceFingerprintHashRequest`
- `SourceFingerprintHashJobInput`
- `SourceFingerprintHashJobSummary`
- `plan_source_fingerprint_hash_scheduling`
- `SourceFingerprintHashExecutor`

`nako-server::app::source_hash` already has:

- internal enqueue for an existing source;
- queued execution planning from persisted `SourceFingerprintHashJobInput`;
- single-job execution through `DurableJobRuntime`;
- claimed-job execution for the disk-scan scheduler;
- persisted redacted hash evidence back onto `MediaSource` and matching
  `SourceState`;
- Admin overview counts.

`nako-server::app::jobs` already lets the disk-scan scheduler claim queued
`JobKind::SourceFingerprintHash` jobs under the `disk.scan` budget. This is
scheduler integration for already queued jobs. It is not a trigger policy.

### Admin And API Surface

Admin currently exposes:

- overview counts for source fingerprint hash source/job state;
- bounded job list filtering by kind, resource class, library id, and source id;
- job cancellation.

Admin does not expose:

- a source fingerprint hash enqueue command;
- a source hash retry/requeue command;
- source hash evidence details;
- duplicate relationship detail or source-level reconciliation plan/apply.

Admin job DTOs expose `has_input`, `has_summary`, and `has_error` booleans.
They do not expose job `input_json`, `summary_json`, raw error payloads, raw
fingerprints, or source locators.

### Duplicate Relationship Persistence

Source Duplicate Relationship records already exist with:

- canonicalized source pair ordering;
- evidence kind;
- optional evidence value;
- status: Suggested, Confirmed, or Rejected;
- optional confidence.

Existing write sources include:

- scan source commits for weak/duplicate fingerprint candidates;
- filesystem link duplicate suggestions;
- managed import duplicate hints.

The hash evidence persistence path does not write duplicate relationships. It
calls `commit_library_scan_source` with an empty
`source_duplicate_relationships` vector when a Source State exists, or updates
only the Media Source when no Source State exists.

One important persistence caveat:

- SQLite baseline has `UNIQUE(source_id, duplicate_source_id)`.
- PostgreSQL baseline has indexes for the pair but no pair-level unique
  constraint.

Any automatic or repeated reconciliation writer must fix or bypass that parity
gap with a repository-level idempotent pair upsert before it can be safe.

## Trigger Options Compared

| Option | Recommendation | Rationale |
| --- | --- | --- |
| Scan-originated enqueue inside source commit | Reject | The source commit planner is pure policy/persistence planning. It must not run VFS reads, enqueue hidden work, or couple scan commit success to hash job creation. |
| Scan-originated enqueue after successful scan commit | Allow in a later policy-backed slice | This can use the existing advisory escalation decision and internal enqueue service, but only after idempotent queue suppression, library policy, and operator visibility are defined. |
| Admin manual enqueue | Recommended first implementation slice | It uses the shipped app service and scheduler path, is easy to authorize and observe, and keeps automatic reconciliation out of scope. |
| Retry/requeue | Reuse durable job semantics; do not create a hidden source-hash loop | The repository has retry primitives, while Admin currently exposes only cancellation. A source hash retry command should validate terminal status, rebuild or reuse safe `SourceFingerprintHashJobInput`, and stay visible in Admin jobs. |
| Policy-backed automatic scheduling | Defer | This needs dedupe, stale guards, per-library enablement, budget interaction, and rollback rules. It should not be default-on. |
| Operator-only diagnostics | Keep and deepen | Overview and job filters are already useful. Evidence/reconciliation details should be added as redacted plan surfaces before any mutation. |

## Recommended Trigger Policy

The next implementation should be Admin-manual triggering only:

- `POST /admin/v1/source-fingerprint-hashes` or an equivalent Admin command
  should enqueue one `JobKind::SourceFingerprintHash` for a specific
  `library_id`, `source_id`, `mode`, and optional priority.
- The handler should delegate to `SourceFingerprintHashAppService`, not create
  jobs directly.
- The response should return a redacted job DTO, not job input JSON or hash
  evidence.
- The existing disk-scan scheduler should remain the executor for queued jobs.
- The command must reject cross-library source IDs and invalid locators without
  echoing the locator.

Scan-originated automatic enqueue should be a later feature:

- It should run after successful scan source commit, not inside
  `plan_source_observation_commit`.
- It should use the existing `SourceFingerprintEscalationDecision`.
- It should be disabled by default until library policy and queue dedupe are
  implemented.
- It should suppress duplicate queued/running work for the same source and mode.
- It should not enqueue if a current non-stale content hash already exists.

Retry/requeue should be a separate Admin command or generic job command:

- It should work only for failed, cancelled, or explicitly stale succeeded
  source hash jobs.
- It should preserve linkage to the prior job when using the durable retry
  model.
- It should not expose or accept raw `StorageUri`, Source Locator, fingerprint,
  hash material, or job input JSON in the Admin request.

## Reconciliation Decision

Duplicate relationship mutation is not in scope for the next implementation
slice.

Persisted hash evidence should not immediately create or update Source
Duplicate Relationship records during source hash job completion. Reasons:

- Source hash execution currently owns byte reading, safe summary, and evidence
  persistence. Adding duplicate writes would mix execution with catalog
  reconciliation policy.
- Automatic relationship mutation needs pair idempotency, staleness checks,
  operator visibility, and undo/reject semantics.
- Existing duplicate writers already come from scan, filesystem link plans, and
  managed import plans. A hash-based reconciler would be another writer and
  must be explicit.
- PostgreSQL currently lacks SQLite's pair-level uniqueness guarantee for
  source duplicate relationships.

The correct shape is a separate reconciliation plan/apply workflow.

## Recommended Reconciliation Policy

Add a future `SourceFingerprintDuplicateReconciliationService` or equivalent
app-service boundary after Admin manual triggering ships.

Read-only plan:

- Input: `library_id`, optional `source_id`, optional mode/evidence filter, page.
- Load current Media Sources and Source States through repository traits.
- Compare only current non-stale redacted fingerprint evidence.
- Same-library reconciliation is the first supported scope.
- Cross-library relationships should remain diagnostics-only until Library
  Access and playback visibility rules are specified.
- Output should include source ids, evidence kind, confidence, stale status,
  existing relationship status, and recommended action.
- Output must not include raw Source Locators, paths, etags, backend URLs,
  credentials, raw hashes, or evidence values.

Apply:

- Create or update Suggested Source Duplicate Relationship records only after a
  plan is explicitly applied by an Admin workflow or a named library policy.
- Do not auto-confirm relationships.
- Do not merge Media Sources.
- Do not merge Media Items.
- Do not change Playback Source Selection.
- Do not change Library Access.
- Canonicalize source pairs before writes.
- Preserve Rejected relationships unless the request explicitly asks to reopen
  them.
- Require pair-level idempotency across SQLite and PostgreSQL before automatic
  writers are allowed.

Staleness and confidence:

- `ContentHash`, non-stale, confidence 1000: may create Suggested
  StrongFingerprint relationships.
- `BackendFingerprint` from partial hash: may create Suggested
  `Other("backend_fingerprint")` relationships only when the plan names the
  weaker confidence and does not imply equality.
- Size/etag or path-only evidence should stay in scan-originated suggestion
  behavior unless a future plan explicitly adds them.
- Stale evidence should produce a hash refresh recommendation, not a duplicate
  write.

Rollback and undo:

- Relationship status changes must be reversible through Suggested,
  Confirmed, and Rejected state.
- Applying a duplicate suggestion should not mutate source rows other than the
  relationship record.
- Rejecting a relationship must not delete source hash evidence.
- Relationship history/audit may be needed before broad automatic scheduling.

## Parallel Conflict Surfaces

Do not run implementation of this policy in parallel with:

- broad durable-job scheduler migration;
- source identity repository/schema work;
- Admin API contract regeneration by another lane;
- scan scheduling/productization work;
- cross-library source visibility or Library Access changes.

Lower-conflict follow-ons:

- Admin manual source hash enqueue route if it owns the Admin contract changes.
- Source duplicate repository idempotent pair upsert parity tests.
- Read-only duplicate reconciliation plan with no apply route.

## First Bounded Follow-on

Recommended first implementation task:

`admin-source-fingerprint-hash-trigger-first-slice`

Scope:

- Add an Admin-only command to enqueue one source fingerprint hash job for a
  known `library_id` and `source_id`.
- Reuse `SourceFingerprintHashAppService::enqueue_source_fingerprint_hash`.
- Return the existing redacted Admin job DTO.
- Add generated Admin contract and route inventory coverage.
- Add HTTP/app tests for admin auth, cross-library rejection, invalid locator
  redaction, mode selection, and job list filter visibility.
- Do not add duplicate relationship mutation.
- Do not add automatic scan-originated scheduling.
- Do not expose job input JSON, source locators, raw hashes, or fingerprints.

Recommended second task:

`source-duplicate-reconciliation-plan-first-slice`

Scope:

- Add a read-only plan for hash-backed duplicate relationship suggestions.
- Add pair-level idempotency requirements and SQLite/PostgreSQL parity tests.
- Do not apply mutations until the plan and undo semantics are accepted.

## Acceptance Criteria Review

- Trigger policy: Admin-manual first; scan-originated policy-backed enqueue
  later; no hidden commit-time scheduling.
- Duplicate mutation: out of next implementation slice; split into explicit
  plan/apply with idempotency and undo.
- Automatic reconciliation risks: documented under idempotency, staleness,
  operator visibility, and rollback.
- Conflict surfaces: listed above.
- First follow-on: Admin source fingerprint hash trigger first slice.
