# Quality Guidelines

Use these gates for `crates/nako-server` feature work.

## Test Patterns

- Prefer `#[tokio::test]` for async app, route, storage, and runtime tests.
- Use `#[test]` for pure functions such as parser, playlist, selection, or
  constant-time comparison behavior.
- Route tests use Axum routers plus `tower::ServiceExt`, not a live network
  server, unless an external service fixture is required.
- Tests create isolated data with `tempfile::tempdir()` and in-memory database
  helpers when possible.
- Auth and access tests should assert both status code and public response body
  shape, including `WWW-Authenticate: Bearer` for `401`.
- ADR 0053 requires new list surfaces to stay bounded and paginated rather than
  returning unbounded JSON.
- When playback admission policy is changed, cover immediate rejection, typed
  bounded wait paths such as `HlsStart`/`HlsSupersede`, and the affected app
  flow. Keep wait constants and configured-capacity checks in the resource
  helper layer.
- HLS admission tests must prove ordinary startup rejects unconfigured capacity
  before FFmpeg input staging, waits only within the bounded policy when
  capacity is busy, preserves `HlsSupersede` for replacements, and releases
  acquired permits when staging or runner work fails.
- Process-backed HLS and remux tests must use bounded polling helpers for
  transcode state and artifact readiness instead of immediate assertions. Full
  workspace nextest on Windows can start many fake FFmpeg processes
  concurrently; readiness budgets should cover that startup tail while still
  bounding hangs.
- Durable job resource-class mapping changes must cover
  `runtime_budget_class_for_job_resource_class` with focused server tests,
  especially when a feature-specific persisted resource class maps onto an
  existing runtime budget such as `disk.scan`.
- Internal source fingerprint hash enqueue changes must prove safe job input
  serialization, missing-source rejection, cross-library rejection, and
  locator/path/error-message redaction with focused app tests.
- Internal source fingerprint hash queued execution planner changes must prove
  successful in-memory request recovery, wrong job kind/resource rejection,
  malformed or unsafe input rejection, binding mismatch rejection, locator
  scheme drift rejection, queued job non-mutation, and locator/path/input
  redaction with focused app tests.
- Internal source fingerprint hash durable executor command changes must prove
  a job is claimed through durable lease runtime, completed with a redaction-safe
  `SourceFingerprintHashJobSummary`, no longer claimable after success, and no
  summary JSON contains locator/path/fingerprint/hash material. Keep automatic
  scheduling and API routes out of command-only slices.
- Source fingerprint hash scheduler integration changes must prove
  scheduler-originated execution succeeds through a claimed-job helper,
  unrelated claimable jobs cannot hide disk-scan candidates, cross-kind
  starvation ordering is preserved, execution failures persist redaction-safe
  durable job errors, and existing background scan scheduler behavior remains
  green. Keep API routes, schema changes, evidence persistence, and duplicate
  reconciliation out of scheduler-only slices.
- Admin operator readiness changes for Media Library scan must include
  watch-folder runtime coverage gaps. Realtime-enabled libraries with
  unsupported or missing watch roots should degrade the scan readiness check
  with a typed reason and redacted source reason. Started watcher latest tick
  statuses of `degraded`, `blocked`, or `reconciliation_pending` should
  degrade readiness before coverage gaps; explicitly disabled watcher coverage
  and non-pressure latest ticks must not degrade readiness by themselves.
- Admin operator readiness changes for Media Library scan must also preserve
  the durable `LibraryScan` posture order: runtime/source-hash repair pressure,
  source-hash pending work, failed same-library `LibraryScan` posture,
  queued/running `LibraryScan` posture, never-completed configured libraries,
  watch-folder latest tick pressure, watch-folder coverage gaps, then ready.
  Tests should prove paginated scan-job aggregation and must not expose job
  `input_json`, `summary_json`, errors, paths, Source Locators, filenames, or
  tokens.
- Admin operator readiness recent-execution evidence for Media Library scan
  must reuse the intake action-plan component status, reason, source reason,
  and attention counts. Job evidence must be a deliberately smaller projection
  than `AdminJobListItem`; watch-folder evidence must expose `scan_job_present`
  rather than `scan_job_id`, collapse discovery failures to counts, and prefer
  a tick whose status matches the action-plan source reason when multiple
  library ticks exist.
- Watch-folder intake tests must prove size-only stability fallback, missing
  size evidence staying `Inspecting`, and changed size resetting the candidate
  before scan admission treats it as ready.
- Admin operator readiness changes for Storage must include unresolved VFS cache
  repair pressure. Healthy/no-action repair diagnostics must not degrade
  readiness, but any current unresolved repair target with retryable refresh
  failure or operator-action pressure should degrade storage readiness with
  typed, redaction-safe source reasons and route operators to the repair target
  surface. Do not base readiness only on the latest failure; a newer resolved
  failure must not hide older unresolved repair targets.
- Internal VFS cache repair durable enqueue changes must prove safe job input
  serialization, non-refresh target rejection, queued/running idempotency,
  duplicate detection beyond the first paginated job page, terminal jobs not
  blocking future enqueue, and no backend refresh/list calls during enqueue.
  Keep Admin routes, durable scheduler execution, cache purge/delete/
  invalidation, backend configuration mutation, and library file writes out of
  enqueue-only slices.
- Internal VFS cache repair durable executor changes must claim an explicit job
  id through `DurableJobRuntime`, validate kind/resource/input/bindings, reload
  the current unresolved failure by `VfsCacheRepairJobInput::matches_failure`,
  reuse selected-target refresh authority for backend-touching work, persist
  redaction-safe summary/error state, and reject stale inputs without backend
  calls. Keep Admin routes, automatic scheduler loops, retry/requeue routes,
  purge/delete/invalidation, backend configuration mutation, and library file
  writes out of single-job executor slices.
- Internal VFS cache repair retry changes must create a new queued retry from
  a failed `JobKind::VfsCacheRepair` job only after validating kind, resource
  class, redaction-safe input, library/source bindings, failed status, and a
  current unresolved `refresh_cache` target. The original failed job remains
  audit history, delayed `next_attempt_at` values are canonical UTC RFC3339,
  future retries are not claimable, due retries keep using the existing
  disk-scan scheduler path, and errors/responses must not leak raw URI, path,
  target ref, input JSON, etag, fingerprint, credential, or backend error
  material. Keep Admin routes, purge/delete/invalidation, backend
  configuration mutation, library file writes, and automated repair policy out
  of retry-only slices.
- Internal VFS cache repair automation policy planner changes must stay dry-run
  and classify existing unresolved repair targets into eligible/blocked groups
  under an explicit policy. Disabled policy reports zero eligible targets;
  enabled policy may mark only `refresh_cache` targets eligible. Planner code
  must not enqueue jobs, refresh cache, purge/delete/invalidate cache entries,
  mutate backend configuration, write library files, or expose raw target
  material.
- Recurring VFS cache repair automation runtime changes must stay
  disabled-by-default, run under `RuntimeSupervisor`, reuse the existing dry-run
  planner and explicit automation enqueue command, and enqueue only
  `refresh_cache` durable jobs. Runtime tests must prove disabled startup,
  enabled supervised startup, one-tick enqueue/schedule behavior, duplicate
  queued/running idempotency, blocked non-refresh targets, bounded delay/backoff
  helpers, and redaction-safe failure summaries.
- Admin VFS cache repair manual command changes must accept only opaque
  `target_ref` values or explicit durable job IDs, return only safe job facts
  and summary facts, inherit the existing Admin route guard, and keep automatic
  schedulers, purge/delete/invalidation, backend configuration mutation, and
  library file writes out of the route slice.

## Scenario: Playback Session Admission And Limits

### 1. Scope / Trigger

- Trigger: changing playback session start behavior, remote playback policy
  enforcement, idle playback-session cleanup, playback runtime settings, or any
  Direct Play, Remux, HLS, or renderer startup path that can create playback
  sessions or downstream playback work.
- Purpose: enforce access and cost policy before stream/transcode work starts
  while keeping admission errors typed and redaction-safe.

### 2. Signatures

- App boundary:
  `PlaybackAppService::admit_playback_session_start(principal, source, effective_policy) -> Result<EffectivePlaybackPolicy>`.
- Session creation after pre-admission:
  `PlaybackAppService::create_playback_session_after_admission(request, source) -> Result<PlaybackSessionRecord>`.
- Repository helpers:
  `PlaybackSessionRepository::count_active_playback_sessions() -> Result<u64>`
  and
  `PlaybackSessionRepository::end_idle_playback_sessions(stale_before_ms, ended_at_ms) -> Result<u64>`.
- Runtime settings fields:
  `active_playback_session_limit` and `idle_playback_session_timeout_ms`.

### 3. Contracts

- Playback app service owns admission. HTTP handlers, planner crates, and
  transcode helpers must not duplicate session-limit or bitrate admission.
- Admission checks library/play access, remote playback permission, remote
  bitrate cap, idle-session cleanup, and active-session ceiling before creating
  a new playback session.
- Idle cleanup treats `active`, `paused`, and `cancel_requested` sessions as
  active candidates and terminalizes only rows whose heartbeat or update time
  is older than the configured timeout.
- Active-session count includes `active`, `paused`, and `cancel_requested`
  states. Terminal states do not consume session ceiling.
- Remote playback over `max_remote_bitrate` returns `NakoError::Forbidden`
  before creating playback session, transcode session, or artifact records.
- Active session ceiling denial returns `NakoError::Conflict` before creating
  playback session, transcode session, or artifact records.
- Remux, HLS, and renderer flows that need downstream work must pre-admit once,
  then create the linked playback session through the after-admission helper so
  admission is not skipped or double-counted.
- Admin runtime settings and diagnostics must expose new admission knobs through
  `nako-api` DTOs and regenerated Admin TypeScript contracts.
- The first slice is process-local and count-then-create. Strict cross-request
  or cross-node reservation requires a dedicated repository transaction or
  reservation contract.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Remote source exceeds effective `max_remote_bitrate` | Return `Forbidden` before session/transcode/artifact creation |
| Caller lacks remote playback permission for a remote source | Return `Forbidden` before downstream playback work |
| Active session count is at or above configured ceiling | Return `Conflict` and create no new playback/transcode/artifact rows |
| Stale active or paused sessions are older than timeout | Terminalize them before counting active sessions |
| Fresh heartbeat exists on an otherwise old session | Keep the session active and count it |
| Runtime setting limit or timeout is zero | Reject the settings update as invalid input |
| Admin contract output changes | Regenerate both Admin TypeScript contract outputs from `nako-api` |
| Error body contains source locator, local path, ticket, or FFmpeg command | Treat as a redaction failure |

### 5. Good/Base/Bad Cases

- Good: a Remux request calls app-service admission, rejects at the session
  ceiling, and never starts a transcode session.
- Good: a stale paused session is ended during admission and no longer blocks a
  fresh playback request.
- Base: a local Direct Play request still uses the same session ceiling but does
  not require remote bitrate checks.
- Bad: an HTTP route counts active sessions directly or starts FFmpeg before
  checking remote bitrate and session ceiling.
- Bad: a helper creates a linked playback session by calling the full public
  start path after pre-admission, causing a second admission check against the
  same request.

### 6. Tests Required

- Server app tests for remote bitrate denial, session ceiling rejection, idle
  session reaping, and Remux/HLS/renderer no-orphan downstream work.
- Server Admin route tests for playback runtime settings validation and
  diagnostics round trip.
- API contract tests:
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
- Repository contract tests for SQLite and PostgreSQL adapter parity:
  `cargo nextest run -p nako-db playback_session_active_count_and_idle_reap --no-fail-fast`.
- Cross-crate compile when public DTOs or repository traits change:
  `cargo check -p nako-api -p nako-core -p nako-db -p nako-server --tests`.
- Formatting and whitespace:
  `cargo fmt --all -- --check` and `git diff --check`.

### 7. Wrong vs Correct

#### Wrong

```rust
let session = self.start_remux_source(request).await?;
self.start_playback_session(linked_session_request).await?;
```

This can start downstream state before policy admission and can double-admit
linked playback sessions.

#### Correct

```rust
let effective_policy = self
    .admit_playback_session_start(&principal, &source, None)
    .await?;
let transcode = self
    .start_remux_source_with_policy(request, effective_policy)
    .await?;
self.create_playback_session_after_admission(linked_session_request, source)
    .await?;
```

Admission happens once before expensive work, and linked session creation stays
inside the already-admitted flow.

## Scenario: VFS Cache Repair Durable Enqueue

### 1. Scope / Trigger

- Trigger: adding or changing internal VFS cache repair durable job enqueue
  behavior in `nako-server::app::storage`.
- Purpose: create repair jobs from opaque unresolved repair targets while
  preserving redaction, stored failure authority, and ADR 0053 durable job
  resource policy.

### 2. Signatures

- Service command:
  `StorageDiagnosticsAppService::enqueue_vfs_cache_repair_target(target_ref: &str, priority: Option<JobPriority>) -> Result<EnqueueVfsCacheRepairTargetOutcome>`.
- Outcome:
  `Enqueued(Job)` or `AlreadyQueued(Job)`.
- Durable job:
  `JobKind::VfsCacheRepair`,
  `resource_class = VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS`
  (`storage.vfs.cache_repair`), mapped to the existing `disk.scan` runtime
  budget.
- Input:
  `VfsCacheRepairJobInput { action, source_scheme, operation, failed_at_ms, failure_count, uri_digest, authority }`.

### 3. Contracts

- `target_ref` is an opaque process-keyed repair target reference produced by
  the existing target inventory flow; callers must not submit raw URI/path
  material.
- Enqueue accepts only unresolved targets whose diagnostic recommends
  `refresh_cache`.
- Enqueue persists digest- and authority-based input only. It must not persist
  raw `StorageUri`, local paths, backend URLs, credentials, raw backend errors,
  etags, fingerprints, or cache payloads.
- Enqueue is non-mutating for storage backends: no cache refresh, purge, delete,
  invalidation, backend configuration change, or library file write may happen
  in the enqueue command.
- Idempotency is based on validated input equality for queued/running jobs and
  must scan all paginated durable job results. Terminal jobs do not block future
  enqueue.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Unknown or malformed `target_ref` | Return `NakoError::NotFound` for `vfs_cache_repair_target` |
| Target no longer unresolved | Do not enqueue; target lookup fails through the existing unresolved-target path |
| Diagnostic recommends `refresh_cache` | Persist one queued `VfsCacheRepair` job |
| Diagnostic recommends backend configuration or manual inspection | Return fixed `InvalidInput` text without backend calls |
| Existing queued/running job has matching validated input | Return `AlreadyQueued(existing)` |
| Matching job exists beyond the first `PageRequest::MAX_LIMIT` page | Return `AlreadyQueued(existing)` |
| Existing matching job is terminal | Allow a new enqueue |
| Stored duplicate candidate has malformed input | Ignore it for idempotency; do not echo malformed JSON |

### 5. Good/Base/Bad Cases

- Good: enqueue a retryable `Stat` failure target, persist only the source
  scheme and URI digest, and assert backend stat/list counters stay at zero.
- Base: enqueue the same target twice while the first job is queued or running;
  return the existing job both times.
- Bad: call `refresh_vfs_cache_repair_target` from the enqueue path, persist the
  raw target URI in `input_json`, or check only the first durable job page for
  duplicates.

### 6. Tests Required

- `cargo nextest run -p nako-server vfs_cache_repair_target_enqueue --no-fail-fast`.
- Assert persisted job kind, resource class, priority, library binding,
  source-id absence, and redaction-safe JSON input.
- Assert non-refresh targets return fixed `InvalidInput` text, create no jobs,
  and make no backend calls.
- Assert queued and running jobs block duplicate enqueue, terminal jobs do not,
  and duplicates beyond the first paginated job page are still found.
- Also run the runtime mapping test whenever the resource class mapping changes:
  `cargo nextest run -p nako-server runtime_job_resource_class_mapping_maps_known_jobs_to_budget_classes --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
let input_json = serde_json::to_string(&failure)?;
backend.refresh_stat_cache(&uri).await?;
```

This persists raw URI/error authority and turns enqueue into execution.

#### Correct

```rust
let input = VfsCacheRepairJobInput::from_failure(&failure)?;
store.enqueue_job(NewJob {
    kind: JobKind::VfsCacheRepair,
    resource_class: VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS.to_owned(),
    input_json: Some(serde_json::to_string(&input)?),
    // ...
}).await?;
```

The durable input stays redaction-safe and backend mutation remains in a future
executor boundary.

## Scenario: VFS Cache Repair Durable Executor Command

### 1. Scope / Trigger

- Trigger: adding or changing internal execution for queued
  `JobKind::VfsCacheRepair` jobs in `nako-server::app::storage`.
- Purpose: execute one explicit durable repair job without adding an Admin API
  route, scheduler loop, retry path, purge/delete behavior, or backend
  configuration workflow.

### 2. Signatures

- Service command:
  `StorageDiagnosticsAppService::execute_vfs_cache_repair_job(job_id: JobId) -> Result<VfsCacheRepairCommandOutput>`.
- Claimed-job helper:
  `StorageDiagnosticsAppService::execute_claimed_vfs_cache_repair_job(LeasedJob) -> Result<VfsCacheRepairCommandOutput>`.
- Summary:
  `VfsCacheRepairJobSummary { action, source_scheme, operation, classification, failure_class, failed_at_ms, failure_count, refreshed_cache_state }`.

### 3. Contracts

- The command claims by exact `job_id`, `JobKind::VfsCacheRepair`, and
  `VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS` through `DurableJobRuntime`.
- Execution validates the persisted input through
  `vfs_cache_repair_job_input_from_job` and verifies the job library/source
  bindings still match the input contract.
- Execution must not reconstruct a raw target URI from durable input. It must
  reload current unresolved cache failures and find the target through
  `VfsCacheRepairJobInput::matches_failure`.
- Backend-touching work reuses the selected-target refresh authority path so
  stored failure authority, backend ambiguity handling, and `refresh_cache`
  recommendation checks stay centralized.
- Successful execution persists a redaction-safe summary JSON. The summary must
  not include raw `StorageUri`, local paths, backend URLs, credentials, raw
  backend errors, etags, fingerprints, cache payloads, or job input JSON.
- If the durable input no longer matches an unresolved repair target, execution
  fails without backend calls and persists only a safe not-found error.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Queued VFS cache repair job has matching unresolved `refresh_cache` target | Claim once, refresh cache through selected-target authority, mark job succeeded, persist safe summary |
| Job kind is not `VfsCacheRepair` | Reject before backend calls |
| Job resource class is not `storage.vfs.cache_repair` | Reject before backend calls |
| Input JSON is missing or malformed | Reject without echoing input JSON |
| Job library binding differs from input authority | Reject without backend calls |
| Job carries a source binding | Reject without backend calls |
| Input no longer matches a current unresolved failure | Mark job failed with safe `vfs_cache_repair_target job_input` not-found error; no backend calls |
| Refresh returns a storage error | Persist redacted storage error using only scheme plus `<redacted>` target |

### 5. Good/Base/Bad Cases

- Good: execute one explicit queued repair job and assert the summary contains
  action, scheme, operation, classification, failure class, failed-at,
  failure-count, and cache state only.
- Base: a future scheduler may pass an already claimed `LeasedJob` to the
  claimed-job helper without re-claiming by id.
- Bad: deserialize `input_json` and synthesize `StorageUri` from the digest,
  call backend refresh directly from the scheduler, or persist backend etag /
  fingerprint / raw URI in `summary_json`.

### 6. Tests Required

- `cargo nextest run -p nako-server vfs_cache_repair_job_executor --no-fail-fast`.
- Assert successful execution marks the job succeeded, makes it no longer
  claimable, refreshes the selected target exactly once, and persists a
  redaction-safe summary.
- Assert stale input fails without backend calls and persists only a safe
  durable error.
- `cargo check -p nako-server --tests`.

### 7. Wrong vs Correct

#### Wrong

```rust
let input: VfsCacheRepairJobInput = serde_json::from_str(job.input_json.as_ref().unwrap())?;
let uri = StorageUri::parse(&input.uri_digest)?;
backend.refresh_cache(&uri, input.operation).await?;
```

This treats a digest like a locator and bypasses the selected-target authority.

#### Correct

```rust
let input = vfs_cache_repair_job_input_from_job(job)?;
let failure = self.vfs_cache_repair_failure_for_job_input(&input).await?;
self.refresh_vfs_cache_repair_failure(failure, ...).await?;
```

The executor uses durable input only to reselect the current unresolved target;
the existing refresh authority owns backend selection and mutation.

## Scenario: Admin VFS Cache Repair Manual Commands

### 1. Scope / Trigger

- Trigger: adding or changing Admin HTTP routes that enqueue or execute
  `JobKind::VfsCacheRepair` jobs in `crates/nako-server::http::admin`.
- Purpose: expose operator-controlled durable repair commands without adding an
  automatic scheduler, retry path, purge/delete behavior, backend configuration
  workflow, or raw storage identity surface.

### 2. Signatures

- Enqueue route:
  `POST /admin/v1/storage/vfs-cache/repair/targets/{target_ref}/jobs`.
- Execute route:
  `POST /admin/v1/storage/vfs-cache/repair/jobs/{job_id}/execute`.
- Enqueue app command:
  `StorageDiagnosticsAppService::enqueue_vfs_cache_repair_target(target_ref, priority)`.
- Execute app command:
  `StorageDiagnosticsAppService::execute_vfs_cache_repair_job(job_id)`.

### 3. Contracts

- Both routes must live under `admin::routes()` so they inherit the existing
  authenticated Admin principal guard.
- Enqueue must accept only the opaque selected-target `target_ref` path value
  and optional priority. It must not accept raw `StorageUri`, URI digests, local
  paths, backend URLs, errors, etags, fingerprints, cache payloads, or durable
  job input JSON from the caller.
- Enqueue delegates to the storage app service, so target lookup, refresh-only
  recommendation checks, idempotency, safe durable input, and non-mutating
  behavior stay centralized.
- Execute accepts only an explicit durable `JobId` and delegates to the storage
  app service, so claiming, input validation, current target reselection, and
  selected-target refresh authority stay centralized.
- Responses may expose only `AdminJobListItem`, enqueue outcome, and
  `AdminVfsCacheRepairJobSummary`. They must not expose raw job input JSON,
  summary JSON, storage URIs, local paths, backend URLs, credentials, raw
  backend errors, etags, fingerprints, URI digests, or cache payloads.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Refreshable unresolved target is enqueued | Return `202 Accepted`, `enqueued`, and safe job facts |
| Matching queued/running job already exists | Return `202 Accepted`, `already_queued`, and the existing safe job facts |
| Non-refresh target is enqueued | Return fixed invalid-input error without creating jobs or leaking the target ref |
| Unknown, malformed, stale, or resolved target ref is enqueued | Return not found without echoing the supplied ref or raw URI |
| Queued repair job is executed | Return safe job facts plus redaction-safe summary |
| Non-admin principal calls either route | Return the existing Admin `403` response |

### 5. Good/Base/Bad Cases

- Good: enqueue a refreshable target, return a safe generic job row, then
  execute that job and return only action, scheme, operation, classification,
  failure class, failed-at, failure-count, and refreshed cache state.
- Base: use the generic Admin Jobs route for broader queue drilldown; the manual
  command routes do not need a raw job detail endpoint.
- Bad: accept a raw URI or durable input JSON in the request body, execute a job
  by deserializing input in the HTTP handler, or return raw `summary_json`.

### 6. Tests Required

- `cargo nextest run -p nako-server admin_v1_vfs_cache --no-fail-fast`.
- Assert enqueue success, duplicate enqueue, non-refresh rejection, and
  unknown/malformed target rejection.
- Assert execute success returns only safe summary facts and marks the job
  succeeded through the durable runtime path.
- Assert non-admin callers are rejected for both routes.
- Also run Admin route inventory and API contract checks whenever route
  constants or DTOs change:
  `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
  and `cargo nextest run -p nako-api admin_contract --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
async fn execute(Json(input): Json<VfsCacheRepairJobInput>) -> ApiResult<_> {
    let uri = StorageUri::parse(&input.uri_digest)?;
    app.storage().refresh_uri(uri).await
}
```

This accepts durable internals from the caller and treats a digest like a
locator.

#### Correct

```rust
async fn execute(State(app): State<NakoApp>, Path(job_id): Path<JobId>) -> ApiResult<_> {
    let output = app.storage().execute_vfs_cache_repair_job(job_id).await?;
    Ok(Json(admin_vfs_cache_repair_execute_response(output)))
}
```

The route is a thin Admin boundary; the app service owns claim, validation,
target reselection, backend selection, mutation, and summary redaction.

## Scenario: Recurring VFS Cache Repair Automation Runtime

### 1. Scope / Trigger

- Trigger: adding or changing startup/runtime behavior that periodically
  discovers unresolved VFS cache repair targets and queues repair jobs.
- Purpose: provide an operator-controlled recurring enqueue policy without
  adding a second repair executor or widening VFS mutation semantics.

### 2. Signatures

- Config:
  `VfsCacheRepairAutomationRuntimeConfig { enabled, interval_ms, error_backoff_ms }`.
- Startup report:
  `ServerStartupReport::vfs_cache_repair_automation_started: bool`.
- Runtime service:
  `VfsCacheRepairAutomationRuntimeAppService::start_recurring_automation(config, runtime) -> bool`.
- Runtime tick:
  `VfsCacheRepairAutomationRuntimeAppService::run_vfs_cache_repair_automation_tick(config) -> Result<VfsCacheRepairAutomationRuntimeTickDiagnostic>`.
- Enqueue authority reused by the runtime:
  `StorageDiagnosticsAppService::enqueue_vfs_cache_repair_automation(VfsCacheRepairAutomationPolicy { enabled: true }, Some(JobPriority::Low))`.
- Execution trigger reused by the runtime:
  `LibraryScanAppService::schedule_queued_library_scans() -> Result<LibraryScanScheduleOutcome>`.

### 3. Contracts

- The runtime is disabled by default. Startup starts it only when
  `vfs_cache_repair_automation.enabled = true`.
- The recurring runtime reads unresolved repair targets only through the
  existing storage automation command. Do not duplicate target inventory,
  classification, target-ref parsing, or durable enqueue logic in the runtime.
- The runtime may enqueue only `refresh_cache` repair jobs through the existing
  automation policy. Non-refresh targets remain blocked by the storage service.
- The runtime must reuse durable queue idempotency; queued/running matching jobs
  are reported as `already_queued` and must not produce duplicate rows.
- The runtime asks the existing disk-scan scheduler to consider queued or
  already queued repair work. Actual repair execution stays in
  `VfsCacheRepair` durable jobs under the `disk.scan` budget.
- The runtime must not call storage backends, refresh cache, purge/delete/
  invalidate cache entries, mutate backend configuration, write library files,
  or create a feature-specific executor.
- Tick logs and diagnostics may expose enabled state, aggregate eligible/
  blocked/enqueued/already-queued counts, scheduler outcome, boundary booleans,
  and safe error summaries only.
- Tick logs, diagnostics, and persisted evidence must not expose raw
  `StorageUri`, local path, backend URL, credential, target ref, URI digest,
  etag, fingerprint, cache payload, job input JSON, or raw backend error text.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Config omits `vfs_cache_repair_automation` | Runtime config defaults to disabled with bounded interval/backoff defaults |
| Config explicitly enables runtime | Startup reports `vfs_cache_repair_automation_started = true` and registers one supervised runtime task |
| Config disables runtime | Startup reports `false` and registers no VFS automation runtime task |
| Enabled tick finds one refreshable target | Queue one low-priority `VfsCacheRepair` job through storage automation and ask the disk-scan scheduler to schedule work |
| Enabled tick finds the same target already queued/running | Report `already_queued`, create no duplicate job, and still ask the disk-scan scheduler to consider queued work |
| Enabled tick finds non-refresh or blocked targets only | Report blocked counts, enqueue no jobs, and do not schedule repair work |
| Planner/enqueue/scheduler returns an error | Loop sleeps for `error_backoff_ms.max(1)` and logs only a safe error summary |
| Interval or backoff is zero | Runtime delay helper clamps it to at least one millisecond |

### 5. Good/Base/Bad Cases

- Good: a supervised runtime tick calls the existing storage automation enqueue
  command, queues one safe `VfsCacheRepair` job, and then calls
  `schedule_queued_library_scans`.
- Base: an existing queued/running repair job makes the tick report
  `already_queued` without changing priority or inserting another durable row.
- Bad: rebuilding target selection in the runtime, refreshing the target
  directly from the runtime, or introducing `vfs_cache_repair_worker` as a
  second executor outside the disk-scan scheduler.

### 6. Tests Required

- Config test: TOML defaults keep the runtime disabled and explicit
  `enabled`, `interval_ms`, and `error_backoff_ms` values parse correctly.
- Startup tests: disabled startup reports no runtime task; enabled startup
  reports `vfs_cache_repair_automation_started` and a supervised
  `vfs_cache_repair_automation_runtime` task with resource class
  `storage.vfs.cache_repair.automation`.
- Runtime tick tests: disabled tick performs no planning/enqueueing; enabled
  tick enqueues and schedules refreshable targets; second tick over
  queued/running work reports `already_queued`; blocked/non-refresh targets do
  not enqueue.
- Redaction tests: runtime safe-error summaries omit raw URI/path/token/backend
  details.
- Focused gates:
  `cargo nextest run -p nako-server vfs_cache_repair_automation --no-fail-fast`,
  `cargo nextest run -p nako-server vfs_cache_repair_scheduler --no-fail-fast`,
  and `cargo nextest run -p nako-server startup --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
runtime.spawn("vfs_cache_repair_worker", "storage.vfs.cache_repair", async move {
    backend.refresh_stat_cache(&raw_uri).await?;
});
```

This adds a second executor, touches the backend from the runtime, and risks
logging or persisting raw storage identity.

#### Correct

```rust
let report = storage
    .enqueue_vfs_cache_repair_automation(
        VfsCacheRepairAutomationPolicy { enabled: true },
        Some(JobPriority::Low),
    )
    .await?;
let _ = library_scan.schedule_queued_library_scans().await?;
```

The runtime remains an enqueue/scheduler trigger. Storage owns target
classification and durable input; the existing disk-scan scheduler owns
execution.

## Scenario: Internal VFS Cache Repair Retry Seam

### 1. Scope / Trigger

- Trigger: adding or changing internal app-service retry behavior for failed
  `JobKind::VfsCacheRepair` jobs in `nako-server::app::storage`.
- Purpose: preserve failed repair jobs as audit history while creating a new
  durable queued retry that keeps all VFS cache repair redaction and current
  target validation rules.

### 2. Signatures

- Service command:
  `StorageDiagnosticsAppService::retry_vfs_cache_repair_job(RetryVfsCacheRepairJobRequest) -> Result<Job>`.
- Request:
  `RetryVfsCacheRepairJobRequest { job_id, max_attempts, next_attempt_at }`.
- Durable retry:
  `JobRepository::enqueue_job_retry(EnqueueJobRetry { source_job_id, retry_job_id, max_attempts, next_attempt_at })`.

### 3. Contracts

- Retry may be exposed through a dedicated Admin route slice only after that
  slice owns the public DTO, route inventory, auth guard, and response
  redaction contract.
- The source job must be `JobKind::VfsCacheRepair`, use
  `VFS_CACHE_REPAIR_JOB_RESOURCE_CLASS`, contain a valid
  `VfsCacheRepairJobInput`, have no source binding, and have a library binding
  matching the input authority when the input is library-scoped.
- Only failed VFS cache repair jobs can be retried. Queued, running, canceled,
  or succeeded jobs must not create retry rows.
- Retry must reselect a current unresolved failure with
  `VfsCacheRepairJobInput::matches_failure`; stale durable inputs fail without
  backend calls.
- Retry delegates durable row creation to `JobRepository::enqueue_job_retry`.
  It must not reset the failed source job in place or bypass generic durable
  retry attempt validation.
- Omitted `max_attempts` uses `max(source.max_attempts, source.attempt + 1)`.
- `next_attempt_at`, when present, must parse as RFC3339 and be persisted as
  canonical UTC RFC3339 so queue ordering remains lexicographic.
- Future-dated retries remain queued but are not claimable until due. Due
  retries continue through the existing disk-scan scheduler path.

### 4. Tests Required

- `cargo nextest run -p nako-server vfs_cache_repair_retry --no-fail-fast`.
- Assert a delayed retry copies safe input/resource/priority/library binding,
  points `retry_of_job_id` at the failed source job, increments attempt, stores
  canonical UTC `next_attempt_at`, and is not claimable before it is due.
- Assert a due retry is scheduled through `schedule_queued_library_scans`,
  executes through the VFS cache repair executor, and persists only the
  redaction-safe summary.
- Assert queued/non-failed, wrong-kind, malformed-input, stale-input,
  exhausted-attempt, invalid-`max_attempts`, and invalid-`next_attempt_at`
  cases create no retry row and do not leak raw URI, path, target ref, input
  JSON, etag, fingerprint, credential, or backend error material.
- Also run source-hash retry tests when touching shared retry timestamp
  helpers: `cargo nextest run -p nako-server source_fingerprint_hash_retry --no-fail-fast`.

## Gate Selection

- Narrow server change:
  `cargo check -p nako-server --tests`
- Focused server behavior:
  `cargo nextest run -p nako-server <filter> --no-fail-fast`
- Cross-crate API/server change:
  `cargo check -p nako-api -p nako-server --tests`
  and focused `nako-api` + `nako-server` nextest filters.
- Full Rust closeout:
  `cargo fmt --all -- --check`, `cargo check --workspace --tests`,
  `cargo nextest run --workspace --no-fail-fast`.

## Scenario: M1 Operator Journey Smoke Gate

### 1. Scope / Trigger

- Trigger: changing `scripts/m1-operator-journey-smoke.ps1`, changing the
  meaning of Product-Operator M1 smoke coverage, or moving the focused server,
  Admin Web, or docs-safe gates that script composes.
- Purpose: provide one repeatable M1 smoke entry point that maps the operator
  journey to existing deterministic gates without publishing release artifacts
  or adding runtime behavior.

### 2. Signatures

- PowerShell gate:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-operator-journey-smoke.ps1 [-Mode docs|server|admin-web|fast] [-SkipDocsGate]`.
- Default mode: `fast`.
- `docs` mode runs only the docs-safe release gate.
- `server` mode runs docs-safe release gate plus `scripts/self-host-smoke.ps1`.
- `admin-web` mode runs docs-safe release gate plus the focused Admin Web
  route/media smoke tests.
- `fast` mode runs docs-safe release gate, server self-host smoke, and focused
  Admin Web route/media smoke tests.

### 3. Contracts

- The script composes existing gates; it must not duplicate server smoke,
  release-gate, or Admin Web route/media test logic inline.
- The smoke maps Product-Operator M1 to concrete checks for configured Media
  Library visibility, scan/index visibility, browse/source inventory, playback
  readiness, Admin diagnostics/repair, and redaction.
- The script must not introduce schema changes, generated contract changes,
  API route shape changes, hidden runtime behavior, release artifact
  publication, automatic source hash scheduling, or automatic duplicate merge
  behavior.
- `-SkipDocsGate` is for focused local iteration only. Task evidence must state
  when it was used, and closeout should run a mode that includes the docs gate.
- Script output may print safe command names and coverage categories, but must
  not print bearer tokens, playback tickets, local paths, source locators,
  playback output paths, database URLs, source fingerprints, content hashes, or
  secret environment values.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Invalid `-Mode` value | PowerShell parameter validation rejects it before any gate runs |
| Docs-safe release gate fails | Script exits non-zero and does not run later gates in that mode |
| Server self-host smoke fails | Script exits non-zero and reports the failed step name |
| Admin Web route/media smoke fails | Script exits non-zero and reports the failed step name |
| A future change needs API/schema/generated-contract/runtime behavior | Stop and open a focused Trellis task instead of expanding the smoke script |
| A future gate needs live browser/manual evidence | Document it as an additional mode or follow-on; do not hide it under `fast` without explicit evidence |
| Output includes unsafe token/path/locator/secret material | Treat as a redaction failure and fix before closeout |

### 5. Good/Base/Bad Cases

- Good: add a focused `playback` mode that invokes an existing playback gate
  script and updates the coverage map, while keeping `fast` deterministic.
- Base: update the script to point at a renamed Admin Web focused test and run
  `-Mode fast` plus task validation.
- Bad: copy `self_host_smoke` setup into the script, call a live server with an
  inline token, publish package/container artifacts, or add route/schema
  behavior to make the smoke pass.

### 6. Tests Required

- `python ./.trellis/scripts/task.py validate <m1-smoke-task-dir>`.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-operator-journey-smoke.ps1 -Mode docs`.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-operator-journey-smoke.ps1 -Mode fast` for closeout when local Rust, Node, and Admin Web tooling are available.
- PowerShell parser check if a focused environment cannot run the full script.
- `git diff --check` after script, docs, task, or spec changes.

### 7. Wrong vs Correct

#### Wrong

```powershell
Write-Host "Token: $env:NAKO_ADMIN_TOKEN"
cargo run -p nako-server -- serve --publish-artifacts
```

The M1 smoke must not echo credentials or turn a validation gate into a release
publication command.

#### Correct

```powershell
Invoke-Step 'scripts/self-host-smoke.ps1' {
    pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/self-host-smoke.ps1
}
```

Delegate to an existing focused gate and let that gate own its detailed setup,
assertions, and redaction behavior.

## Scenario: M1 Release Ladder Runner

### 1. Scope / Trigger

- Trigger: changing `scripts/m1-release-ladder.ps1`, changing the meaning of
  Product-Operator M1 release confidence, or changing how M1 composes
  release-gate and operator-smoke scripts.
- Purpose: provide one product-level validation entry point while preserving
  `scripts/release-gate.ps1` and `scripts/m1-operator-journey-smoke.ps1` as
  the owners of detailed gate logic.

### 2. Signatures

- PowerShell runner:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 [-Mode docs|smoke|fast|release-fast|playback|container|postgres|workspace|all] [-PostgresUrl <url>] [-SkipRedactionInventory]`.
- Default mode: `fast`.
- `fast` runs `release-gate.ps1 -Mode docs` once, then runs
  `m1-operator-journey-smoke.ps1 -Mode fast -SkipDocsGate`.
- `release-fast`, `playback`, `container`, `postgres`, and `workspace`
  delegate to the matching `release-gate.ps1` mode.
- `all` sequences fast, release-fast, playback, container, postgres, and
  workspace; after the first docs gate it skips repeated redaction inventory
  scans.
- `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md` records the release-facing
  evidence meaning, tooling requirements, skipped-gate rules, and follow-up
  routing for every runner mode.

### 3. Contracts

- The runner is an orchestrator only. It must not copy the command list from
  `release-gate.ps1`, `self-host-smoke.ps1`, Admin Web tests, or playback
  release gates.
- Default `fast` mode must remain suitable for local M1 confidence. Expensive
  modes must stay explicit.
- The runner may print mode names, safe command names, and coverage categories.
  It must not print PostgreSQL URLs, bearer tokens, playback tickets, local
  media paths, Source Locators, playback output paths, raw fingerprints,
  content hashes, or secret environment values.
- Adding live-browser, package-publication, or release-artifact validation must
  be explicit new scope. Do not hide those steps under `fast`.
- Any runner mode addition, removal, rename, or semantic change must update
  `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`,
  `docs/deployment/RELEASE_CHECKLIST.md`, and
  `docs/architecture/OPERATIONS_RELEASE.md` in the same task.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Invalid `-Mode` value | PowerShell parameter validation rejects it before any gate runs |
| Docs release gate fails | Runner exits non-zero before M1 smoke in `fast` mode |
| M1 operator smoke fails | Runner exits non-zero and reports the failed step name |
| Expensive release-gate mode fails | Runner exits non-zero and reports the delegated mode |
| PostgreSQL URL is provided | Runner passes it to `release-gate.ps1` but prints only `<provided>` |
| Output includes unsafe URL/token/path/locator/secret material | Treat as a redaction failure and fix before closeout |

### 5. Good/Base/Bad Cases

- Good: add a future `live-browser` mode that invokes a dedicated browser smoke
  script and keeps default `fast` unchanged.
- Base: update the runner to call a renamed M1 smoke script and validate
  `-Mode docs` plus parser checks.
- Bad: paste the `release-gate.ps1` command list into the ladder, echo
  `$PostgresUrl`, publish artifacts, or add live browser work to default
  `fast`.

### 6. Tests Required

- PowerShell parser validation for `scripts/m1-release-ladder.ps1`.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode docs -SkipRedactionInventory`.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode fast -SkipRedactionInventory` for closeout when local Rust, Node, and Admin Web tooling are available.
- `rg -n "docs|smoke|fast|release-fast|playback|container|postgres|workspace|all" scripts/m1-release-ladder.ps1 docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md`
  or an equivalent review proving the matrix covers every runner mode.
- `python ./.trellis/scripts/task.py validate <m1-release-ladder-task-dir>`.
- `git diff --check`.

### 7. Wrong vs Correct

#### Wrong

```powershell
Write-Host "Postgres: $PostgresUrl"
cargo nextest run -p nako-server self_host_smoke --no-fail-fast
```

The runner must not print sensitive URLs or copy a detailed gate that already
has an owning script.

#### Correct

```powershell
Invoke-Step 'scripts/release-gate.ps1 -Mode playback' {
    pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode playback
}
```

Delegate to the existing release gate and keep M1 ladder logic at the product
orchestration layer.

## Scenario: Remote Access Config Gate Fixtures

### 1. Scope / Trigger

- Trigger: changing remote access cookbook docs, `deploy/remote-access/*.toml`,
  `scripts/remote-access-config-gate.*`, or `config-check` network readiness
  output.
- Purpose: prove reverse-proxy and external tunnel-provider examples are
  accepted by `nako-server config-check --json --create-dirs` while raw network
  facts remain redacted.

### 2. Signatures

- PowerShell gate:
  `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/remote-access-config-gate.ps1`.
- Bash gate:
  `bash scripts/remote-access-config-gate.sh`.
- Fixtures:
  `deploy/remote-access/reverse-proxy.nako.toml` and
  `deploy/remote-access/tunnel-provider.nako.toml`.
- Output reports:
  `target/release-gate/remote-access/<fixture>-config-check.json`.
- Expected network check IDs:
  `network.access`, `network.proxy`, `network.origins`, and
  `network.tunnel_providers`.

### 3. Contracts

- Fixtures must keep auth enabled and use loopback/private listener defaults.
- `reverse_proxy` fixtures must use HTTPS `external_base_url`, exact
  `allowed_origins`, and explicit reviewed `trusted_proxy_sources`.
- `tunnel_provider` fixtures must declare external provider metadata only; they
  must not start, supervise, or configure tunnel processes.
- Gate scripts must set fixture-only environment variables for auth/tunnel
  tokens and restore the caller environment afterward.
- JSON reports must not contain raw fixture URLs, tunnel token values, bearer
  token values, private origins, trusted proxy sources, forwarded header names,
  or local host details such as `127.0.0.1`.
- Config-only CLI commands such as `config-example` and `config-check` must
  complete before server runtime startup. The remote access gate is an operator
  preflight path, not a reason to construct `NakoApp` or enter the full async
  command future.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Config-check exits non-zero | Gate fails and leaves no successful report for that fixture |
| Overall report status is not `pass` | Gate fails |
| Expected network check is missing or not `pass` | Gate fails |
| Report contains fixture URL, origin, proxy source, header name, token, or local host detail | Gate fails |
| Tunnel provider config implies process/runtime ownership | Reject the docs/fixture change or move it to a dedicated architecture task |
| Bash cannot run in the current environment | At least `bash -n` must pass and the reason actual execution could not run must be recorded |

### 5. Good/Base/Bad Cases

- Good: add a new Cloudflare Tunnel fixture that declares provider kind and
  `token_env`, then update both gate scripts with the expected checks and
  redaction assertions.
- Base: cookbook docs add provider guidance without changing fixtures; run
  `git diff --check` and the PowerShell gate.
- Bad: adding a tunnel supervisor, endpoint discovery route, wildcard CORS
  origin, or raw `public_url` echo in config-check output.

### 6. Tests Required

- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/remote-access-config-gate.ps1`.
- `bash -n scripts/remote-access-config-gate.sh`.
- `cargo run -q -p nako-server -- config-example` when changing CLI runtime
  dispatch for config-only commands.
- Actual Bash gate when the shell environment has working Cargo/Rust.
- `python .trellis/scripts/task.py validate <remote-access-task-dir>`.
- `git diff --check`.
- `cargo fmt --all -- --check` only when Rust code changes.

### 7. Wrong vs Correct

#### Wrong

```toml
[network]
exposure_mode = "tunnel_provider"
external_base_url = "https://nako.example.com?token=secret"
```

Embedding provider or bearer secrets in URLs makes config-check, logs, and
support bundles harder to redact.

#### Correct

```toml
[[network.tunnel_providers]]
id = "cloudflared"
kind = "cloudflare_tunnel"
public_url = "https://nako.example.com"
token_env = "NAKO_TUNNEL_TOKEN"
```

Tunnel credentials stay in environment-backed operator secrets; Nako records
only readiness declarations and redacted diagnostics.

## Forbidden Patterns

- Do not add unauthenticated sensitive routes outside the explicit public route
  groups in `http.rs`.
- Do not return raw domain/database records from HTTP handlers.
- Do not log secrets, raw tokens, playback tickets, or local filesystem paths.
- Do not rely on `cargo test` as the default Rust gate when `cargo nextest` is
  available; this repo has `.config/nextest.toml` and CI installs nextest.

## Evidence

- `.config/nextest.toml`
- `.github/workflows/release-gate.yml`
- `crates/nako-server/src/http/tests/mod.rs`
- `crates/nako-server/src/app/tests/*.rs`
