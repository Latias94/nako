# Source Fingerprint Hash Triggering And Reconciliation

Use this spec before changing server-side source fingerprint hash triggers,
Admin source hash commands, scan-originated source hash scheduling, or duplicate
relationship reconciliation from hash evidence.

## Scenario: Source Fingerprint Hash Triggering And Reconciliation

### 1. Scope / Trigger

- Trigger: adding Admin/API source fingerprint hash enqueue, retry/requeue,
  scan-originated hash scheduling, source hash evidence detail diagnostics, or
  duplicate relationship reconciliation from persisted source hash evidence,
  including read-only Admin reconciliation plan routes and explicit Admin
  reconciliation apply routes.
- Scope: `nako-server` owns app-service orchestration and Admin/HTTP mapping;
  `nako-library::source_hash` owns hash request/input/summary and execution
  contracts; `nako-library::ingestion` owns source observation advisory
  escalation decisions.

### 2. Signatures

- Internal enqueue:
  `SourceFingerprintHashAppService::enqueue_source_fingerprint_hash(EnqueueSourceFingerprintHashRequest) -> Result<Job>`.
- Scan-originated internal enqueue:
  `SourceFingerprintHashAppService::enqueue_scan_originated_source_fingerprint_hash(
  library_id, &ScanSourceFingerprintHashTrigger,
  ScanOriginatedSourceFingerprintHashPolicy
  ) -> Result<ScanOriginatedSourceFingerprintHashOutcome>`.
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
- Scan source trigger fact:
  `ScanSourceFingerprintHashTrigger { source_id, decision, mode }`.
- Scan summary internal trigger channel:
  `LibraryIndexSummary.source_fingerprint_hash_triggers:
  Vec<ScanSourceFingerprintHashTrigger>` with `#[serde(skip)]`.
- Internal read-only duplicate reconciliation:
  `SourceDuplicateReconciliationAppService::plan_source_duplicate_reconciliation(
  SourceDuplicateReconciliationPlanRequest { library_id, source_id, page }
  ) -> Result<SourceDuplicateReconciliationPlan>`.
- Admin read-only duplicate reconciliation:
  `GET /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan`
  with `limit` / `offset` pagination, returning
  `AdminSourceDuplicateReconciliationPlanResponse`.
- Internal duplicate reconciliation apply:
  `SourceDuplicateReconciliationAppService::apply_source_duplicate_reconciliation(
  SourceDuplicateReconciliationApplyRequest { library_id, source_id,
  duplicate_source_id, expected_action }
  ) -> Result<SourceDuplicateReconciliationApplyResult>`.
- Admin duplicate reconciliation apply:
  `POST /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply`
  with `AdminSourceDuplicateReconciliationApplyRequest {
  duplicate_source_id, expected_action }`, where `expected_action` is limited
  to `suggest_relationship`, returning
  `AdminSourceDuplicateReconciliationApplyResponse`.
- Durable input:
  `SourceFingerprintHashJobInput { library_id, source_id, source_scheme, mode,
  request_id? }`.
- Admin job list:
  `GET /admin/v1/jobs` may filter by `kind`, `resource_class`,
  `library_id`, and `source_id`.
- Admin job diagnostics:
  `AdminJobDiagnostics { source_fingerprint_hash:
  AdminSourceFingerprintHashJobDiagnostics | null }`.
- Admin source hash diagnostics:
  `AdminSourceFingerprintHashJobDiagnostics { status, summary, failure }`,
  where `status` is `pending`, `summary_available`, or `failed`.
- Admin source hash summary:
  `AdminSourceFingerprintHashJobSummary { mode, evidence_kind,
  confidence_milli, stale, bytes_hashed }`, where `mode` is normalized to
  `{ mode, prefix_bytes? }`.
- Admin source hash failure:
  `AdminSourceFingerprintHashJobFailureDiagnostic { status, safe_message,
  retryable }`.
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
- Generated Admin contract keys:
  `sourceDuplicateReconciliationPlan` maps to
  `/admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan`;
  `sourceDuplicateReconciliationApply` maps to
  `/admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply`.

### 3. Contracts

- Source Fingerprint is evidence, not Source identity. Hash execution must not
  merge Media Sources or Media Items.
- Source observation planning may produce
  `SourceFingerprintEscalationDecision`; it must not perform VFS reads or
  enqueue durable jobs inside the pure planning function.
- Library indexing may aggregate committed source trigger facts in
  `LibraryIndexSummary.source_fingerprint_hash_triggers`, but that field is an
  in-process server channel only and must stay `#[serde(skip)]`. Public scan
  command summaries and event payloads must not expose trigger facts, Source
  Locators, fingerprints, or job input.
- Scan-originated scheduling runs only after `nako-library::ingestion`
  successfully commits a source observation and returns a stable
  `MediaSourceId`. `nako-server::app::jobs` must call the source-hash app
  service after indexing succeeds and before probe/metadata work; scan planning
  itself remains pure.
- Scan-originated policy is explicit:
  `ScanOriginatedSourceFingerprintHashPolicy { enabled, partial_prefix_bytes,
  priority }` returns advisory-only when `enabled = false`; otherwise it maps
  partial/full decisions to durable source hash jobs. A zero partial prefix is
  invalid input.
- The default scan-originated policy is enabled with
  `DEFAULT_SCAN_SOURCE_FINGERPRINT_HASH_PARTIAL_PREFIX_BYTES` and
  `JobPriority::Normal`.
- Scan-originated enqueue must delegate to
  `SourceFingerprintHashAppService::enqueue_source_fingerprint_hash` after its
  own policy and idempotency checks. It must not insert durable jobs directly
  from library scan code. When a trace context is present, it should be copied
  into the durable request_id for correlation.
- Scan-originated idempotency must check existing queued and running
  `JobKind::SourceFingerprintHash` jobs for the same library, source, resource
  class, and hash mode across every job-list page. If one exists, return
  `AlreadyQueued(job)` and do not enqueue another job. Terminal jobs may be
  followed by a new scan-originated enqueue.
- Admin manual enqueue reuses the shipped internal enqueue and disk-scan
  scheduler execution path. The HTTP handler must translate the Admin request
  into `EnqueueSourceFingerprintHashRequest`, normalize the current HTTP trace
  context into a durable request_id, and delegate to
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
  timestamps, `has_input`/`has_summary`/`has_error` booleans, and the optional
  typed `diagnostics.source_fingerprint_hash` branch. They must not expose
  durable input JSON, raw summary JSON, raw errors, Source Locators, local
  paths, etags, backend URLs, credentials, raw hashes, or fingerprints.
- Admin retry responses must use the same redacted Admin job DTO surface as
  enqueue: job id, kind, status, resource class, optional library/source
  bindings, timestamps, `has_input`/`has_summary`/`has_error` booleans, and
  the optional typed `diagnostics.source_fingerprint_hash` branch. They must
  not expose retry linkage (`retry_of_job_id`), priority, attempt/max-attempt
  counters, durable input JSON, raw summary JSON, raw errors, Source Locators,
  local paths, etags, backend URLs, credentials, raw hashes, fingerprints, or
  fingerprint evidence values beyond the explicit safe summary fields listed
  in this spec. Retry linkage and attempt counters may be verified only through
  persisted durable job records or a future Admin Jobs drilldown surface that
  explicitly owns that detail.
- Admin Jobs source hash diagnostics may expose only:
  - pending state for jobs without safe summary or error;
  - parsed safe summary fields from `SourceFingerprintHashJobSummary`;
  - a generic failed diagnostic with `safe_message =
    "source fingerprint hash failed"` and retryability.
- Admin summary parsing must tolerate persisted library summary mode JSON
  (`"full"` or `{"partial":{"prefix_bytes":...}}`) and normalized Admin JSON
  (`{"mode":"partial","prefix_bytes":...}`), but Admin output must use the
  normalized shape.
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
- The Admin reconciliation apply HTTP handler must remain thin: parse path IDs
  and the Admin request body, delegate to
  `SourceDuplicateReconciliationAppService::apply_source_duplicate_reconciliation`,
  and map through explicit `nako_api::admin` DTO conversion. It must not inspect
  repositories directly or decide candidate freshness in HTTP.
- Explicit reconciliation apply may create a new
  `SourceDuplicateRelationshipStatus::Suggested` relationship only when the
  current fresh plan action for the exact source pair is
  `suggest_relationship`. Replaying the same apply may return the existing
  Suggested relationship with `created: false`; it must not create a second row
  or overwrite the existing pair.
- Explicit reconciliation apply must not overwrite existing Suggested,
  Confirmed, or Rejected relationships, auto-confirm duplicates, merge sources,
  change Playback Source Selection, or mutate Library Access. Existing
  Confirmed and Rejected pairs remain preserved until a separate reopen/undo
  workflow is specified.
- Admin reconciliation apply responses may expose Admin API version, library id,
  source id, duplicate source id, relationship id/status, applied action, and
  `created`. They must not expose raw Source Locators, local paths, etags,
  backend URLs, credentials, raw hashes, raw fingerprints, evidence values,
  source fingerprint material, or job input JSON.

### 4. Validation & Error Matrix

| Condition | Behavior |
| --- | --- |
| Admin trigger uses `partial` without `partial_prefix_bytes` | Return invalid input before enqueue. |
| Admin trigger uses `partial_prefix_bytes = 0` | Return invalid input before enqueue. |
| Admin trigger uses `full` with `partial_prefix_bytes` | Return invalid input before enqueue. |
| Admin trigger uses missing source id | Return not found; do not enqueue. |
| Admin trigger source belongs to a different library | Return invalid input without locator/path details. |
| Source locator is malformed | Return invalid input without echoing the locator. |
| Scan escalation action is `none` | Return advisory-only; do not enqueue source hash work. |
| Scan escalation action is partial/full but automatic policy is disabled | Return advisory-only; do not enqueue source hash work. |
| Scan partial escalation uses an enabled policy with `partial_prefix_bytes = 0` | Return invalid input before enqueue. |
| Scan partial/full escalation has an equivalent queued/running source hash job for the same source/mode | Return `AlreadyQueued(job)`; do not enqueue a duplicate job. |
| Scan partial/full escalation has only terminal prior source hash jobs for the same source/mode | A new scan-originated enqueue may create fresh queued work. |
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
| Admin Jobs lists a queued/running source hash job with no summary/error | Return pending source hash diagnostics with null summary and failure. |
| Admin Jobs lists a succeeded source hash job with valid summary JSON | Return summary-available diagnostics with normalized safe summary fields. |
| Admin Jobs lists a source hash job with malformed summary JSON | Do not expose the raw summary or parse error; omit the summary branch. |
| Admin Jobs lists a failed source hash job | Return failed diagnostics with a generic safe message and no raw error body. |
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
| Admin apply request uses an `expected_action` other than `suggest_relationship` | Return invalid input and do not write a relationship. |
| Admin apply target and duplicate source are the same Media Source | Return invalid input and do not write a relationship. |
| Admin apply target source is missing | Return not found without locator/path details. |
| Admin apply candidate source is missing | Return not found without locator/path details. |
| Admin apply target source belongs to another library | Return invalid input without locator/path details. |
| Admin apply candidate source belongs to another library | Return invalid input without locator/path details. |
| Admin apply target or candidate has no fingerprint | Return invalid input without locator/path details. |
| Admin apply target or candidate has a raw or unsupported fingerprint string | Return invalid input without echoing the fingerprint. |
| Admin apply target and candidate fingerprints do not match | Return invalid input without echoing either fingerprint. |
| Admin apply target or candidate evidence is stale | Return conflict recommending `refresh_source_fingerprint`; do not write a relationship. |
| Admin apply current pair already has Suggested status | Return the existing relationship with `created: false`; do not insert or update a row. |
| Admin apply current pair already has Confirmed status | Return conflict preserving `preserve_confirmed`; do not update the row. |
| Admin apply current pair already has Rejected status | Return conflict preserving `preserve_rejected`; do not update the row. |
| Admin apply caller is non-admin | Return forbidden through the existing Admin route guard. |
| PostgreSQL lacks pair-level uniqueness for relationships | Do not enable automatic reconciliation writers. |

### 5. Good / Base / Bad Cases

- Good: an Admin command enqueues a full source hash job by library/source id,
  the existing scheduler executes it under `disk.scan`, and Admin jobs show a
  redacted row with `has_input`/`has_summary` booleans plus typed safe
  diagnostics.
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
- Good: a succeeded source hash job surfaces mode, evidence kind, confidence,
  stale state, and bytes hashed in `diagnostics.source_fingerprint_hash.summary`
  without returning raw summary JSON or fingerprint material.
- Good: a failed source hash job surfaces a generic failed diagnostic without
  returning durable error text, Source Locators, paths, etags, hashes,
  fingerprints, tokens, or credentials.
- Good: an immediate retry is executed by
  `LibraryScanAppService::schedule_queued_library_scans` through the same
  disk-scan source-hash path as a freshly queued source hash job.
- Good: a successful scan source commit returns a partial escalation trigger;
  `LibraryIndexSummary` carries the redaction-safe trigger in-process only;
  the server enqueues exactly one partial source hash job with safe durable
  input, source/library bindings, and `disk.scan.source_fingerprint_hash`.
- Good: replaying the same scan while the same source/mode hash job is queued
  or running returns an idempotent `AlreadyQueued` outcome and leaves only one
  incomplete source hash job.
- Good: a later read-only reconciliation plan reports that two sources share a
  non-stale content hash and recommends a Suggested relationship without
  writing anything.
- Good: a read-only reconciliation plan preserves existing Suggested,
  Confirmed, and Rejected relationship statuses while recommending refresh for
  stale fingerprint evidence.
- Good: the Admin reconciliation route exposes the same read-only plan with
  `AdminSourceDuplicateReconciliationPlanResponse`, generated Admin contract
  route key, and no raw fingerprint/locator material.
- Good: the Admin reconciliation apply route explicitly applies one fresh
  `suggest_relationship` candidate and returns a redaction-safe
  `AdminSourceDuplicateReconciliationApplyResponse` with `created: true`.
- Good: replaying the same Admin apply returns the existing Suggested
  relationship with `created: false` and does not insert another row.
- Good: applying against stale evidence or an existing Confirmed/Rejected
  relationship returns a safe conflict and preserves stored relationship state.
- Base: scan commit records a partial/full hash advisory decision but leaves
  scheduling disabled, returning advisory-only and creating no job.
- Base: scan command JSON and `LibraryScanned` event payloads omit
  `source_fingerprint_hash_triggers`; operators observe scan-originated work
  through the existing Admin Jobs/source hash surfaces.
- Bad: `persist_source_fingerprint_hash_evidence` also calls
  `upsert_source_duplicate_relationship`.
- Bad: source commit planning starts a VFS full hash read during scan commit.
- Bad: library indexing serializes `source_fingerprint_hash_triggers` into a
  public scan summary or durable event payload.
- Bad: `LibraryScanAppService` inserts `NewJob` rows directly instead of
  delegating to the source-hash app service.
- Bad: Admin DTOs expose job input JSON, Source Locators, raw fingerprints, or
  raw hash material.
- Bad: Admin Jobs renders `summary_json` or job error text directly for source
  hash diagnostics instead of parsing into the safe DTO.
- Bad: the Admin apply handler queries repositories directly or duplicates
  fingerprint freshness/action policy instead of delegating to the app service.
- Bad: explicit apply relies on repository upsert to overwrite an existing
  Confirmed or Rejected relationship.
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
- Library index tests proving committed source observations aggregate
  redaction-safe `ScanSourceFingerprintHashTrigger` facts and that public
  summary serialization skips the trigger channel.
- Scan-originated app-service tests proving disabled policy, `none` advisory
  decisions, partial/full enqueue, configured partial prefix/priority, queued
  and running idempotency, and durable input redaction.
- Server scan integration tests proving a successful scan with a weak
  same-library fingerprint match enqueues exactly one source fingerprint hash
  job through the scan pipeline without exposing locators, paths, raw
  fingerprints, or job input in scan-facing surfaces.
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
- Admin Jobs source hash diagnostic tests proving pending, succeeded-summary,
  failed, malformed-summary, generated contract, and HTTP filter responses
  expose only the safe diagnostics branch and never raw input JSON, summary
  JSON, Source Locators, paths, etags, backend URLs, credentials, raw hashes,
  raw fingerprints, tokens, or raw error bodies.
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
- Reconciliation apply app-service tests proving fresh apply creates one
  Suggested relationship with canonical source pair ordering, replay preserves
  the existing Suggested row with `created: false`, existing Confirmed and
  Rejected relationships are not overwritten, stale evidence recommends refresh
  without writing, mismatched fingerprints are rejected, and all errors/results
  remain redaction-safe.
- Admin apply route tests proving success, replay idempotency, Admin guard,
  generated route key coverage, safe missing/cross-library/missing-fingerprint/
  raw-fingerprint/mismatched-fingerprint/stale errors, existing
  Suggested/Confirmed/Rejected preservation, no duplicate row creation, and
  response redaction.
- Gate for Admin trigger:
  `cargo check -p nako-api -p nako-server --tests` plus focused
  `cargo nextest run -p nako-server source_fingerprint_hash --no-fail-fast`
  and `cargo nextest run -p nako-api admin_contract --no-fail-fast` when route
  inventory changes.
- Gate for scan-originated triggering:
  `cargo check -p nako-library --tests`,
  `cargo check -p nako-server --bin nako-server --tests`,
  `cargo nextest run -p nako-library index_service_returns_redacted_source_hash_trigger_facts --no-fail-fast`,
  `cargo nextest run -p nako-server scan_originated_source_fingerprint_hash --no-fail-fast`,
  `cargo nextest run -p nako-server scan_library_enqueues_scan_originated_source_hash_after_weak_match --no-fail-fast`,
  `cargo nextest run -p nako-server source_fingerprint_hash --no-fail-fast`,
  `cargo fmt --all -- --check`, `git diff --check`, and Trellis task
  validation.
- Gate for Admin retry route:
  `cargo check -p nako-api -p nako-server --tests`,
  `cargo nextest run -p nako-server source_fingerprint_hash --no-fail-fast`,
  `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`,
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`,
  `cargo fmt --all -- --check`, `git diff --check`, and Trellis task
  validation.
- Gate for Admin reconciliation route:
  `cargo nextest run -p nako-server admin_v1_source_duplicate_reconciliation --no-fail-fast`,
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
