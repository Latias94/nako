# Source Fingerprint Hash Triggering And Reconciliation

Use this spec before changing server-side source fingerprint hash triggers,
Admin source hash commands, scan-originated source hash scheduling, or duplicate
relationship reconciliation from hash evidence.

## Scenario: Source Fingerprint Hash Triggering And Reconciliation

### 1. Scope / Trigger

- Trigger: adding Admin/API source fingerprint hash enqueue, retry/requeue,
  scan-originated hash scheduling, source hash evidence detail diagnostics, or
  duplicate relationship reconciliation from persisted source hash evidence,
  including read-only Admin reconciliation plan routes.
- Scope: `nako-server` owns app-service orchestration and Admin/HTTP mapping;
  `nako-library::source_hash` owns hash request/input/summary and execution
  contracts; `nako-library::ingestion` owns source observation advisory
  escalation decisions.

### 2. Signatures

- Internal enqueue:
  `SourceFingerprintHashAppService::enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest) -> Result<Job>`.
- Internal retry:
  `SourceFingerprintHashAppService::retry_source_fingerprint_hash_job(
  RetrySourceFingerprintHashRequest { job_id, max_attempts, next_attempt_at }
  ) -> Result<Job>`.
- Admin manual enqueue:
  `POST /admin/v1/source-fingerprint-hashes` with
  `AdminSourceFingerprintHashEnqueueRequest { library_id, source_id, mode,
  partial_prefix_bytes, priority }`, returning the redacted
  `AdminJobListItem` shape with `202 Accepted`.
- Admin retry:
  `POST /admin/v1/source-fingerprint-hashes/jobs/{job_id}/retry` with
  `AdminSourceFingerprintHashRetryRequest { max_attempts, next_attempt_at }`,
  returning the redacted `AdminJobListItem` shape with `202 Accepted`.
- Execution:
  `SourceFingerprintHashAppService::execute_source_fingerprint_hash_job(JobId)
  -> Result<SourceFingerprintHashCommandOutput>`.
- Scheduler execution:
  `LibraryScanAppService::schedule_queued_library_scans() ->
  Result<LibraryScanScheduleOutcome>`.
- Internal read-only duplicate reconciliation:
  `SourceDuplicateReconciliationAppService::plan_source_duplicate_reconciliation(
  SourceDuplicateReconciliationPlanRequest { library_id, source_id, page }
  ) -> Result<SourceDuplicateReconciliationPlan>`.
- Admin read-only duplicate reconciliation:
  `GET /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan`
  with `limit` / `offset` pagination, returning
  `AdminSourceDuplicateReconciliationPlanResponse`.
- Durable input:
  `SourceFingerprintHashJobInput { library_id, source_id, source_scheme, mode }`.
- Admin job list:
  `GET /admin/v1/jobs` may filter by `kind`, `resource_class`,
  `library_id`, and `source_id`.
- Admin trigger request mode is `full` or `partial`. `partial_prefix_bytes`
  is required only for `partial`, must be greater than zero, and is rejected
  for `full`. Optional priority maps to durable job priority `low`,
  `normal`, or `high`.
- Admin retry request fields:
  - `max_attempts: Option<u32>` overrides the retry job maximum attempts.
  - `next_attempt_at: Option<String>` delays the retry until an RFC3339
    timestamp. The app service must persist it as canonical UTC RFC3339 with
    a `Z` offset so durable queue claim and overview comparisons remain
    lexicographically valid.
  - If `max_attempts` is omitted, the app service must use
    `max(source.max_attempts, source.attempt + 1)`.
- Generated Admin contract key:
  `sourceFingerprintHashJobRetry` maps to
  `/admin/v1/source-fingerprint-hashes/jobs/{job_id}/retry`.

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
- Admin retry is source-hash-specific. The HTTP handler must translate path
  and body fields into `RetrySourceFingerprintHashRequest`, require the
  existing Admin route guard, delegate to
  `SourceFingerprintHashAppService::retry_source_fingerprint_hash_job`, and
  return `AdminJobListItem`. It must not call generic durable retry directly
  from HTTP.
- Source hash retry must reuse `JobRepository::enqueue_job_retry` semantics:
  create a new queued retry job, preserve the failed source job as audit
  history, set `retry_of_job_id` to the failed job, increment the attempt
  number, copy the safe durable input, resource class, priority, library id,
  and source id, and honor optional `next_attempt_at`.
- Delayed retry jobs remain durable queued jobs but are not claimable until
  `next_attempt_at` is due. Source hash overview must count delayed retry
  pressure and expose only redacted queue timing, such as `delayed_retry_jobs`
  and `next_retry_at`.
- Immediate retry jobs must continue through the existing disk-scan scheduler
  path; retry must not add a separate execution path or bypass the
  source-hash runtime.
- Persisted source fingerprint hash jobs must use
  `SourceFingerprintHashJobInput`; do not persist `StorageUri`, Source Locator,
  path, etag, backend URL, credential, raw digest, or fingerprint material.
- Admin enqueue responses must use the existing redacted Admin job DTO surface:
  job id, kind, status, resource class, optional library/source bindings,
  timestamps, and `has_input`/`has_summary`/`has_error` booleans only. They
  must not expose durable input JSON, summary JSON, raw errors, Source
  Locators, local paths, etags, backend URLs, credentials, raw hashes, or
  fingerprints.
- Admin retry responses must use the same redacted Admin job DTO surface as
  enqueue: job id, kind, status, resource class, optional library/source
  bindings, timestamps, and `has_input`/`has_summary`/`has_error` booleans
  only. They must not expose retry linkage (`retry_of_job_id`), priority,
  attempt/max-attempt counters, durable input JSON, summary JSON, raw errors,
  Source Locators, local paths, etags, backend URLs, credentials, raw hashes,
  fingerprints, or fingerprint evidence values. Retry linkage and attempt
  counters may be verified only through persisted durable job records or an
  Admin Jobs drilldown surface that explicitly owns that detail.
- Source hash retry must validate the failed job contract before retrying:
  `JobKind::SourceFingerprintHash`, resource class
  `SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS`, valid
  `SourceFingerprintHashJobInput`, job library/source bindings matching the
  input, the referenced Media Source still belonging to the input library, and
  the current Source Locator scheme still matching `input.source_scheme`.
- Source hash job completion may persist redacted fingerprint evidence back to
  `MediaSource` and matching `SourceState`.
- Source hash job completion must not directly write
  `SourceDuplicateRelationship` records. Duplicate reconciliation belongs to a
  separate plan/apply service.
- Read-only duplicate reconciliation must load the requested Media Source,
  recover the persisted redacted fingerprint evidence kind, read bounded
  same-library fingerprint matches excluding the target source before
  pagination, read existing canonical pair relationships, and return
  `SourceDuplicateReconciliationPlan` without writing
  `SourceDuplicateRelationship` records.
- Duplicate reconciliation must canonicalize source pairs and require pair-level
  idempotency across SQLite and PostgreSQL before any repeated or automatic
  writer is enabled.
- Reconciliation plans and Admin reconciliation DTOs may expose `library_id`,
  target `source_id`, fingerprint evidence kind, confidence, stale state,
  candidate `source_id`, candidate `duplicate_source_id`, candidate evidence
  kind, candidate confidence, candidate stale state, relationship id, existing
  relationship status, recommended action, and page metadata. They must not
  expose raw Source Locators, local paths, etags, backend URLs, credentials, raw
  hashes, raw fingerprints, evidence values, source fingerprint material, or
  job input JSON.
- The Admin reconciliation HTTP handler must remain thin: parse path IDs and
  bounded pagination, delegate to `SourceDuplicateReconciliationAppService`,
  and map through explicit `nako_api::admin` DTO conversion. It must not read
  repositories directly or mutate duplicate relationships.
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
| Admin retry caller is non-admin | Return forbidden through the existing Admin route guard. |
| Admin retry uses missing job id | Return not found without job input, locator, path, or raw error details. |
| Admin retry job kind is not `SourceFingerprintHash` | Return invalid input; do not create a retry job. |
| Admin retry job resource class is not `SOURCE_FINGERPRINT_HASH_JOB_RESOURCE_CLASS` | Return invalid input; do not create a retry job. |
| Admin retry target job is not failed | Return conflict; only failed source hash jobs can be retried. |
| Admin retry target job has exhausted attempts and no higher `max_attempts` is supplied | Return conflict through durable retry semantics; do not mutate the failed job. |
| Admin retry target job has missing durable input | Return invalid input without exposing job input. |
| Admin retry target job has malformed durable input JSON | Return invalid input without echoing raw input. |
| Admin retry durable input fails `SourceFingerprintHashJobInput::new` validation | Return invalid input without echoing source scheme, locator, or raw input. |
| Admin retry job library binding differs from durable input library id | Return invalid input without retrying. |
| Admin retry job source binding differs from durable input source id | Return invalid input without retrying. |
| Admin retry input source id no longer exists | Return not found without locator/path details. |
| Admin retry input source now belongs to another library | Return conflict without locator/path details. |
| Admin retry current source locator is malformed | Return invalid input without echoing the locator. |
| Admin retry current source locator scheme differs from `input.source_scheme` | Return conflict without echoing either locator. |
| Admin retry `next_attempt_at` is not RFC3339 | Return invalid input without echoing the supplied value. |
| Admin retry `next_attempt_at` uses a non-UTC RFC3339 offset | Accept it, convert it to canonical UTC RFC3339 with a `Z` offset, and persist only the canonical value. |
| Admin retry omits `max_attempts` | Use `max(source.max_attempts, source.attempt + 1)`. |
| Admin retry supplies future `next_attempt_at` | Create a queued retry job that is visible in overview as delayed and cannot be claimed until due. |
| Admin retry supplies due or past `next_attempt_at` | Create a queued retry job that the existing scheduler may execute through the disk-scan source-hash path. |
| Scan escalation action is `none` | Do not enqueue source hash work. |
| Scan escalation action is partial/full but automatic policy is disabled | Keep diagnostics/advisory only. |
| Hash execution succeeds | Persist redacted source fingerprint evidence and redacted job summary. |
| Hash execution fails with a storage error | Persist a redacted durable job error using only a synthetic scheme URI. |
| Reconciliation uses missing source id | Return not found without locator/path details. |
| Reconciliation source belongs to a different library | Return invalid input without locator/path details. |
| Reconciliation source has no fingerprint | Return invalid input without locator/path details. |
| Reconciliation source has a raw or unsupported fingerprint string | Return invalid input without echoing the fingerprint. |
| Admin reconciliation caller is non-admin | Return forbidden through the existing Admin route guard. |
| Admin reconciliation uses `limit` or `offset` | Apply existing Admin bounded pagination before candidate response mapping. |
| Reconciliation sees stale evidence | Recommend hash refresh; do not write duplicate relationships. |
| Reconciliation sees non-stale content hash match | Plan a Suggested StrongFingerprint relationship, not a confirmed merge. |
| Reconciliation sees partial/backend fingerprint match | Plan a weaker Suggested relationship and label the evidence kind/confidence. |
| Existing relationship is Suggested or Confirmed | Preserve existing status in the read-only plan. |
| Existing relationship is Rejected | Preserve Rejected unless an explicit reopen/apply request is provided. |
| PostgreSQL lacks pair-level uniqueness for relationships | Do not enable automatic reconciliation writers. |

### 5. Good / Base / Bad Cases

- Good: an Admin command enqueues a full source hash job by library/source id,
  the existing scheduler executes it under `disk.scan`, and Admin jobs show a
  redacted row with `has_input`/`has_summary` booleans only.
- Good: an Admin command enqueues a partial source hash job only when the
  request includes a positive `partial_prefix_bytes`; the persisted durable
  input stores mode and source scheme but not a Source Locator.
- Good: an Admin command retries a failed source hash job and receives a new
  queued retry job whose `retry_of_job_id` points at the failed job; the failed
  job remains failed for audit.
- Good: an Admin retry copies safe source hash input, resource class, priority,
  library id, and source id from the failed job, increments the attempt, and
  does not expose input JSON or raw error bodies in the response.
- Good: a future-dated retry is visible in source hash overview as delayed
  retry pressure and is not claimable until due; offset `next_attempt_at`
  input is stored and reported as canonical UTC `Z` time.
- Good: an immediate retry is executed by
  `LibraryScanAppService::schedule_queued_library_scans` through the same
  disk-scan source-hash path as a freshly queued source hash job.
- Good: a later read-only reconciliation plan reports that two sources share a
  non-stale content hash and recommends a Suggested relationship without
  writing anything.
- Good: a read-only reconciliation plan preserves existing Suggested,
  Confirmed, and Rejected relationship statuses while recommending refresh for
  stale fingerprint evidence.
- Good: the Admin reconciliation route exposes the same read-only plan with
  `AdminSourceDuplicateReconciliationPlanResponse`, generated Admin contract
  route key, and no raw fingerprint/locator material.
- Base: scan commit records a partial/full hash advisory decision but leaves
  scheduling disabled.
- Bad: `persist_source_fingerprint_hash_evidence` also calls
  `upsert_source_duplicate_relationship`.
- Bad: source commit planning starts a VFS full hash read during scan commit.
- Bad: Admin DTOs expose job input JSON, Source Locators, raw fingerprints, or
  raw hash material.
- Bad: the Admin retry handler calls generic durable retry directly and skips
  source-hash kind/resource/input/binding/source-scheme validation.
- Bad: retry resets the failed job in place instead of creating a new queued
  retry job with `retry_of_job_id`.
- Bad: a delayed retry can be claimed before `next_attempt_at`.
- Bad: automatic reconciliation writes duplicate pairs repeatedly on PostgreSQL
  because pair-level uniqueness/idempotency was not enforced.

### 6. Tests Required

- Admin trigger app/HTTP tests for enqueue success, auth/admin guard,
  cross-library rejection, invalid locator redaction, mode selection, priority,
  and job-list filter visibility.
- Scheduler regression tests proving queued source hash jobs still execute
  through `schedule_queued_library_scans` and keep the `disk.scan` budget.
- Source hash retry app-service tests proving safe retry creation from a failed
  job, original failed job preservation, copied safe input/resource/priority/
  library/source bindings, `retry_of_job_id`, attempt/max-attempt behavior,
  delayed retry overview pressure, future retries not claimable, due retries
  executable through `schedule_queued_library_scans`, and response/input
  redaction.
- Source hash retry validation tests for wrong kind, wrong resource class,
  non-failed job, exhausted attempts, missing input, malformed input, invalid
  source hash input, mismatched library/source bindings, missing source,
  library drift, locator parse failure, source locator scheme drift, invalid
  `next_attempt_at`, and non-admin HTTP caller rejection.
- Admin retry route tests proving `202 Accepted` success, generated route key
  coverage, Admin guard, safe error statuses/messages, no retry job on invalid
  states, copied retry metadata, and redaction of job input, locators, paths,
  etags, backend URLs, credentials, raw hashes, raw fingerprints, and raw error
  bodies.
- Redaction tests proving trigger responses, job rows, errors, and summaries do
  not contain Source Locators, paths, raw digests, raw fingerprints, etags,
  backend URLs, credentials, or job input JSON.
- Reconciliation plan tests for content hash, backend fingerprint, stale
  evidence, existing Suggested/Confirmed/Rejected relationships, and
  same-library scoping.
- SQLite and PostgreSQL repository tests for idempotent pair-level duplicate
  relationship upsert before any automatic or repeated reconciliation writer is
  enabled.
- Repository contract tests for bounded same-library fingerprint match reads,
  stale source-state projection, pagination, and canonical pair lookup.
- App-service tests proving read-only reconciliation does not mutate
  relationships and redacts locator/path/etag/raw fingerprint material.
- Admin route tests proving success, admin guard, safe missing/cross-library/
  missing-fingerprint/raw-fingerprint errors, candidate-oriented pagination,
  read-only relationship state, and response redaction.
- Gate for Admin trigger:
  `cargo check -p nako-api -p nako-server --tests` plus focused
  `cargo nextest run -p nako-server source_fingerprint_hash --no-fail-fast`
  and `cargo nextest run -p nako-api admin_contract --no-fail-fast` when route
  inventory changes.
- Gate for Admin retry route:
  `cargo check -p nako-api -p nako-server --tests`,
  `cargo nextest run -p nako-server source_fingerprint_hash --no-fail-fast`,
  `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`,
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`,
  `cargo fmt --all -- --check`, `git diff --check`, and Trellis task
  validation.
- Gate for Admin reconciliation route:
  `cargo nextest run -p nako-server admin_v1_source_duplicate_reconciliation_plan --no-fail-fast`,
  `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`,
  and `cargo nextest run -p nako-api admin_contract --no-fail-fast`.

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

#### Wrong

```rust
async fn retry_admin_source_fingerprint_hash_job(job_id: JobId) -> Result<Job> {
    self.jobs.enqueue_job_retry(EnqueueJobRetry {
        source_job_id: job_id,
        retry_job_id: JobId::new(),
        max_attempts: 3,
        next_attempt_at: None,
    }).await
}
```

This exposes generic retry behavior without proving the job is a safe Source
Fingerprint Hash job or that its stored input still matches the current Source.

#### Correct

```rust
async fn retry_source_fingerprint_hash_job(
    request: RetrySourceFingerprintHashRequest,
) -> Result<Job> {
    let failed = self.job_for_hash(request.job_id).await?;
    validate_source_fingerprint_hash_job_contract(&failed)?;
    let input = source_fingerprint_hash_job_input_from_job(&failed)?;
    validate_source_fingerprint_hash_job_bindings(&failed, &input)?;
    self.validate_source_fingerprint_hash_retry_source(&input).await?;
    let next_attempt_at = canonical_retry_next_attempt(&request.next_attempt_at)?;

    self.store.enqueue_job_retry(EnqueueJobRetry {
        source_job_id: failed.id,
        retry_job_id: JobId::new(),
        max_attempts: request.max_attempts
            .unwrap_or_else(|| failed.max_attempts.max(failed.attempt + 1)),
        next_attempt_at,
    }).await
}
```

The source-hash app service owns retry validation, while the repository owns
durable retry creation and claimability semantics.
