# Source Fingerprint Hash Triggering And Reconciliation

Use this spec before changing server-side source fingerprint hash triggers,
Admin source hash commands, scan-originated source hash scheduling, or duplicate
relationship reconciliation from hash evidence.

## Scenario: Source Fingerprint Hash Triggering And Reconciliation

### 1. Scope / Trigger

- Trigger: adding Admin/API source fingerprint hash enqueue, retry/requeue,
  scan-originated hash scheduling, source hash evidence detail diagnostics, or
  duplicate relationship reconciliation from persisted source hash evidence.
- Scope: `nako-server` owns app-service orchestration and Admin/HTTP mapping;
  `nako-library::source_hash` owns hash request/input/summary and execution
  contracts; `nako-library::ingestion` owns source observation advisory
  escalation decisions.

### 2. Signatures

- Internal enqueue:
  `SourceFingerprintHashAppService::enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest) -> Result<Job>`.
- Admin manual enqueue:
  `POST /admin/v1/source-fingerprint-hashes` with
  `AdminSourceFingerprintHashEnqueueRequest { library_id, source_id, mode,
  partial_prefix_bytes, priority }`, returning the redacted
  `AdminJobListItem` shape with `202 Accepted`.
- Execution:
  `SourceFingerprintHashAppService::execute_source_fingerprint_hash_job(JobId)
  -> Result<SourceFingerprintHashCommandOutput>`.
- Scheduler execution:
  `LibraryScanAppService::schedule_queued_library_scans() ->
  Result<LibraryScanScheduleOutcome>`.
- Durable input:
  `SourceFingerprintHashJobInput { library_id, source_id, source_scheme, mode }`.
- Admin job list:
  `GET /admin/v1/jobs` may filter by `kind`, `resource_class`,
  `library_id`, and `source_id`.
- Admin trigger request mode is `full` or `partial`. `partial_prefix_bytes`
  is required only for `partial`, must be greater than zero, and is rejected
  for `full`. Optional priority maps to durable job priority `low`,
  `normal`, or `high`.

### 3. Contracts

- Source Fingerprint is evidence, not Source identity. Hash execution must not
  merge Media Sources or Media Items.
- Source observation planning may produce
  `SourceFingerprintEscalationDecision`; it must not perform VFS reads or
  enqueue durable jobs inside the pure planning function.
- Scan-originated scheduling, if added later, must run after a successful scan
  source commit through a server app service and must be policy-backed,
  idempotent, and visible through Admin jobs.
- Admin manual enqueue reuses the shipped internal enqueue and disk-scan
  scheduler execution path. The HTTP handler must translate the Admin request
  into `EnqueueSourceFingerprintHashRequest` and delegate to
  `SourceFingerprintHashAppService`; it must not create durable jobs directly.
- Persisted source fingerprint hash jobs must use
  `SourceFingerprintHashJobInput`; do not persist `StorageUri`, Source Locator,
  path, etag, backend URL, credential, raw digest, or fingerprint material.
- Admin enqueue responses must use the existing redacted Admin job DTO surface:
  job id, kind, status, resource class, optional library/source bindings,
  timestamps, and `has_input`/`has_summary`/`has_error` booleans only. They
  must not expose durable input JSON, summary JSON, raw errors, Source
  Locators, local paths, etags, backend URLs, credentials, raw hashes, or
  fingerprints.
- Source hash job completion may persist redacted fingerprint evidence back to
  `MediaSource` and matching `SourceState`.
- Source hash job completion must not directly write
  `SourceDuplicateRelationship` records. Duplicate reconciliation belongs to a
  separate plan/apply service.
- Future duplicate reconciliation must canonicalize source pairs and require
  pair-level idempotency across SQLite and PostgreSQL before any repeated or
  automatic writer is enabled.
- Future Admin reconciliation DTOs may expose source ids, evidence kind,
  confidence, stale state, existing relationship status, and recommended
  action. They must not expose raw Source Locators, local paths, etags,
  backend URLs, credentials, raw hashes, raw fingerprints, or job input JSON.
- Reconciliation apply may create or update Suggested relationships. It must
  not auto-confirm duplicates, merge sources, change Playback Source Selection,
  or mutate Library Access.

### 4. Validation & Error Matrix

| Condition | Behavior |
| --- | --- |
| Admin trigger uses `partial` without `partial_prefix_bytes` | Return invalid input before enqueue. |
| Admin trigger uses `partial_prefix_bytes = 0` | Return invalid input before enqueue. |
| Admin trigger uses `full` with `partial_prefix_bytes` | Return invalid input before enqueue. |
| Admin trigger uses missing source id | Return not found; do not enqueue. |
| Admin trigger source belongs to a different library | Return invalid input without locator/path details. |
| Source locator is malformed | Return invalid input without echoing the locator. |
| Existing queued/running source hash job for same source/mode exists in a future dedupe policy | Return the existing job or an idempotent conflict; do not enqueue duplicates silently. |
| Scan escalation action is `none` | Do not enqueue source hash work. |
| Scan escalation action is partial/full but automatic policy is disabled | Keep diagnostics/advisory only. |
| Hash execution succeeds | Persist redacted source fingerprint evidence and redacted job summary. |
| Hash execution fails with a storage error | Persist a redacted durable job error using only a synthetic scheme URI. |
| Reconciliation sees stale evidence | Recommend hash refresh; do not write duplicate relationships. |
| Reconciliation sees non-stale content hash match | Plan a Suggested StrongFingerprint relationship, not a confirmed merge. |
| Reconciliation sees partial/backend fingerprint match | Plan a weaker Suggested relationship and label the evidence kind/confidence. |
| Existing relationship is Rejected | Preserve Rejected unless an explicit reopen/apply request is provided. |
| PostgreSQL lacks pair-level uniqueness for relationships | Do not enable automatic reconciliation writers. |

### 5. Good / Base / Bad Cases

- Good: an Admin command enqueues a full source hash job by library/source id,
  the existing scheduler executes it under `disk.scan`, and Admin jobs show a
  redacted row with `has_input`/`has_summary` booleans only.
- Good: an Admin command enqueues a partial source hash job only when the
  request includes a positive `partial_prefix_bytes`; the persisted durable
  input stores mode and source scheme but not a Source Locator.
- Good: a later read-only reconciliation plan reports that two sources share a
  non-stale content hash and recommends a Suggested relationship without
  writing anything.
- Base: scan commit records a partial/full hash advisory decision but leaves
  scheduling disabled.
- Bad: `persist_source_fingerprint_hash_evidence` also calls
  `upsert_source_duplicate_relationship`.
- Bad: source commit planning starts a VFS full hash read during scan commit.
- Bad: Admin DTOs expose job input JSON, Source Locators, raw fingerprints, or
  raw hash material.
- Bad: automatic reconciliation writes duplicate pairs repeatedly on PostgreSQL
  because pair-level uniqueness/idempotency was not enforced.

### 6. Tests Required

- Admin trigger app/HTTP tests for enqueue success, auth/admin guard,
  cross-library rejection, invalid locator redaction, mode selection, priority,
  and job-list filter visibility.
- Scheduler regression tests proving queued source hash jobs still execute
  through `schedule_queued_library_scans` and keep the `disk.scan` budget.
- Redaction tests proving trigger responses, job rows, errors, and summaries do
  not contain Source Locators, paths, raw digests, raw fingerprints, etags,
  backend URLs, credentials, or job input JSON.
- Reconciliation plan tests for content hash, backend fingerprint, stale
  evidence, existing Suggested/Confirmed/Rejected relationships, and
  same-library scoping.
- SQLite and PostgreSQL repository tests for idempotent pair-level duplicate
  relationship upsert before any automatic or repeated reconciliation writer is
  enabled.
- Gate for Admin trigger:
  `cargo check -p nako-api -p nako-server --tests` plus focused
  `cargo nextest run -p nako-server source_fingerprint_hash --no-fail-fast`
  and `cargo nextest run -p nako-api admin_contract --no-fail-fast` when route
  inventory changes.

### 7. Wrong vs Correct

#### Wrong

```rust
async fn persist_source_fingerprint_hash_evidence(...) {
    self.store.upsert_media_source(&source).await?;
    self.store.upsert_source_duplicate_relationship(&relationship).await?;
}
```

This couples byte-hash execution to duplicate reconciliation and creates
implicit catalog mutation without plan/apply or undo semantics.

#### Correct

```rust
async fn persist_source_fingerprint_hash_evidence(...) {
    self.store.upsert_media_source(&source).await?;
}
```

```rust
async fn plan_source_duplicate_reconciliation(...) -> Result<Plan> {
    // Read current fingerprint evidence and existing relationships.
    // Return redacted suggested actions; do not mutate.
}
```

Hash completion persists evidence only. A separate reconciliation workflow owns
duplicate relationship suggestions and applies them explicitly.
