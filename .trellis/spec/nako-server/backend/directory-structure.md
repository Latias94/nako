# Directory Structure

`nako-server` is the composition, app orchestration, runtime-supervision, and
HTTP boundary crate. Keep handlers thin and put business workflow in focused app
services.

## Current Layout

```text
crates/nako-server/src/
├── main.rs                # binary entrypoint
├── config.rs              # server config and preflight
├── app.rs                 # NakoApp root and app module exports
├── app/                   # orchestration services and runtime helpers
├── http.rs                # router assembly and auth layering
├── http/                  # Axum route modules and HTTP mapping
├── api_mapping.rs         # domain/API conversion helpers
├── playback_mapping.rs    # playback-specific API conversion helpers
└── *_tests/ or tests/     # app and HTTP test modules
```

## Module Rules

- Put HTTP route handlers under `src/http/<area>.rs`.
- Put workflow logic under `src/app/<area>.rs` or a focused submodule such as
  `src/app/playback/*`.
- Keep HTTP handlers as request/response translators. They should call app
  services instead of directly coordinating database, storage, and runtime work.
- Keep runtime supervisors and durable job orchestration in `src/app/*runtime*`
  modules, not in individual route handlers.
- Keep config parsing and operator preflight in `config.rs` and
  `config/preflight.rs`.
- Put route tests under `src/http/tests/` and app-service tests under
  `src/app/tests/`.
- Prefer a named startup workflow in `src/app/startup.rs` when `NakoApp`
  composition needs to do more than wire dependencies together. `composition.rs`
  should stay the composition root and pass a narrow runtime view into the
  startup workflow instead of rebuilding startup report fields inline.

## Control-Plane Boundary

- Durable jobs, runtime supervision, diagnostics, addon mediation, remote
  access, and API scale contracts are shared control-plane behavior. Check ADR
  0053 before adding hidden per-feature helpers.
- Long-running scan, metadata, playback, addon, webhook, or artifact workflows
  must use durable job/runtime boundaries instead of raw `tokio::spawn`.
- Source fingerprint hash durable jobs use
  `JobKind::SourceFingerprintHash` with
  `disk.scan.source_fingerprint_hash` as the persisted job resource class. Map
  that class to the existing `disk.scan` runtime budget class. The internal
  enqueue service may load the current Media Source, derive only source scheme
  from its locator, and persist `SourceFingerprintHashJobInput`; do not create
  a separate hidden runtime, scheduler, API route, evidence writer, or VFS
  executor during enqueue-only slices.
- The source fingerprint hash queued execution planner may validate a persisted
  `Job`, deserialize `SourceFingerprintHashJobInput`, reload the current Media
  Source, and rebuild an in-memory `SourceFingerprintHashRequest`. It must not
  claim leases, mark jobs terminal, read VFS bytes, expose an API route, persist
  evidence, or log/return raw locator and input JSON content.
- The source fingerprint hash durable executor command may claim one explicit
  job id through `DurableJobRuntime`, reuse the queued execution planner,
  execute the hash through the configured `StorageBackendRegistry`, and persist
  `SourceFingerprintHashJobSummary` as job `summary_json`. This command is not
  a scheduler, startup worker, runtime-supervisor loop, API route, evidence
  writer, or duplicate reconciliation mechanism.
- Source fingerprint hash scheduler integration belongs in the existing disk
  scan scheduler path. It may dispatch already queued
  `JobKind::SourceFingerprintHash` jobs through a claimed-job executor helper,
  but must not create a source-hash-specific runtime loop or re-claim an already
  leased job.
- Resource admission belongs in app runtime helpers such as
  `app/playback/resource.rs`, not in pure planner crates.
- Bounded resource-admission policy (for example immediate vs HLS supersede
  wait) should live in the resource helper layer and be reused by orchestration
  code instead of being duplicated in HLS/remux flow modules.

## Scenario: Source Fingerprint Hash Disk Scan Scheduler

### 1. Scope / Trigger

- Trigger: queued `JobKind::SourceFingerprintHash` work needs automatic
  execution by the server scheduler.
- Scope: `nako-server::app::jobs` owns disk-scan scheduling and
  `nako-server::app::source_hash` owns source hash job execution.

### 2. Signatures

- `LibraryScanAppService::schedule_queued_library_scans() ->
  Result<LibraryScanScheduleOutcome>` is the current disk-scan scheduler tick.
- `SourceFingerprintHashAppService::execute_claimed_source_fingerprint_hash_job(LeasedJob)
  -> Result<SourceFingerprintHashCommandOutput>` is the scheduler-safe
  claimed-job execution entry point.
- `SourceFingerprintHashAppService::execute_source_fingerprint_hash_job(JobId)
  -> Result<SourceFingerprintHashCommandOutput>` remains the manual command
  entry point that claims by explicit job id.

### 3. Contracts

- The scheduler may consider both `JobKind::LibraryScan` with `disk.scan` and
  `JobKind::SourceFingerprintHash` with
  `disk.scan.source_fingerprint_hash`.
- Candidate preview must not use an unfiltered all-job window that can be filled
  by unrelated metadata, addon, webhook, or artifact jobs.
- Cross-kind disk-scan candidate ordering must preserve durable queue priority,
  FIFO, and starvation-guard semantics closely enough that an aged low-priority
  disk-scan job can run before fresh high-priority disk-scan work.
- The scheduler must claim the selected source hash job once, by exact job id,
  kind, resource class, and available bindings, then pass the `LeasedJob` into
  the claimed-job helper. Do not call the id-claiming command after the
  scheduler has already leased the job.
- The supervised job must use `RuntimeSupervisor::spawn_job` and the mapped
  `disk.scan` runtime budget. Keep the scan permit alive until source hashing
  has completed or failed.
- Execution failures must be persisted with redaction-safe errors. Do not let
  raw `StorageUri`, Source Locator, local path, backend URL, credential, etag,
  raw digest, fingerprint, or hash material reach durable job errors, summaries,
  diagnostics, or logs.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Claimable source hash job and scan budget available | Scheduler returns `Scheduled(job_id)`, supervised execution succeeds, and `summary_json` is redaction-safe |
| Source hash job already claimed by another worker | Scheduler skips it and continues looking for another runnable disk-scan candidate |
| First disk-scan scan candidates are storage-admission blocked but source hash work is runnable | Blocked scan jobs stay queued; source hash job may be scheduled |
| Unrelated claimable jobs fill the durable queue preview window | Disk-scan scheduler still finds source hash candidates through filtered disk-scan candidate queries |
| Aged low-priority source hash and fresh high-priority library scan compete | The aged candidate follows starvation-guard ordering |
| VFS execution fails with a path-bearing storage error | Persist a redacted source-hash error; do not echo locator/path/hash material |

### 5. Good / Base / Bad Cases

- Good: list disk-scan candidates by supported kind/resource windows, merge them
  with durable queue ordering, claim the selected job once, and spawn a
  supervised source hash job that calls the claimed executor.
- Base: the manual app command can still execute one explicit source hash job by
  id and claim internally.
- Bad: query all claimable durable jobs with `JobLeaseClaimFilter::default()`
  and hope non-disk jobs do not fill the scheduler candidate limit.
- Bad: scheduler claims a source hash job, then calls a command that tries to
  claim the same job id again.
- Bad: persist backend storage errors that contain local paths, locators,
  credentials, raw hashes, or file names.

### 6. Tests Required

- App test: scheduler-originated source hash execution succeeds, marks the job
  succeeded, persists redaction-safe `summary_json`, and leaves the job no
  longer claimable.
- App test: unrelated claimable jobs cannot fill the preview window and hide a
  queued source hash job.
- App test: cross-kind disk-scan ordering preserves starvation behavior.
- App test: execution failure persists a redaction-safe durable job error.
- Regression tests: existing background scan scheduler tests for blocked
  libraries and budget saturation remain green.

### 7. Wrong vs Correct

#### Wrong

```rust
let candidates = store
    .list_claimable_jobs_for_lease(JobLeaseClaimFilter::default(), page)
    .await?;
```

This lets unrelated durable jobs occupy the scheduler candidate window and can
hide runnable disk-scan work.

#### Correct

```rust
let scans = store.list_claimable_jobs_for_lease(scan_filter, page).await?;
let hashes = store.list_claimable_jobs_for_lease(source_hash_filter, page).await?;
let candidates = merge_disk_scan_candidates(scans, hashes);
```

The disk-scan scheduler owns only disk-scan work while preserving queue ordering
inside its own resource budget.

## Scenario: Playback FFmpeg Input Staging Scope

### 1. Scope / Trigger

- Trigger: changing HLS, Remux, or another server Playback Runtime path that
  gives FFmpeg an input path for a `Media Source`.

### 2. Signatures

- `FfmpegInputService::with_source_input(source, uri, backend, operation) ->
  Result<T>` is the normal scoped entry point. The operation receives only the
  local FFmpeg input `PathBuf`.
- `FfmpegInputService::source_input_scope(...) -> Result<FfmpegSourceInputScope>`
  plus `with_prepared_source_input(scope, operation) -> Result<T>` is reserved
  for flows that must prove staging succeeded before moving execution into a
  supervised background task, such as HLS playlist startup.

### 3. Contracts

- Local path inputs use the backend local path hint and must not create or
  release an FFmpeg input staging manifest lease.
- Remote inputs are staged through `ManifestRecordingStorageBackend`, acquire a
  staging manifest lease, run the scoped operation, then explicitly release the
  lease with async release before returning from the scope.
- On scoped operation success, a release failure is returned to the caller.
- On scoped operation error, a release failure is logged as cleanup trouble and
  the original operation error is returned.
- HLS and Remux flow modules should not call a staging release method directly
  or inspect whether an FFmpeg input owns a lease.
- Do not rely on plain `Drop` as the primary async release mechanism. Drop may
  remain a defensive fallback, but the playback flow must use the scoped async
  interface.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Local backend exposes `local_path_hint` | Operation runs with that path; no `FfmpegInput` staging manifest record is created |
| Remote backend needs staging and operation succeeds | Staging manifest returns to `Ready` with `active_leases = 0`; operation output is returned |
| Remote backend needs staging and operation fails | Staging manifest is released; original operation error is returned |
| Operation succeeds but release fails | Release error is returned |
| Operation fails and release also fails | Release failure is logged; original operation error is returned |
| HLS playlist startup stages input before background start | Staging error is returned synchronously before spawning runner work |

### 5. Good / Base / Bad Cases

- Good: Remux source startup calls `with_prepared_source_input` and runs
  `RemuxAppService::run` inside the closure that receives the input path.
- Good: HLS playlist startup obtains an opaque `FfmpegSourceInputScope`
  synchronously, moves it into the supervised background task, and runs the
  runner through `with_prepared_source_input`.
- Base: test-only helpers may use `with_source_input` to assert path and lease
  behavior directly.
- Bad: returning `FfmpegSourceInput` to HLS or Remux and requiring those modules
  to match success/error paths and call `release_source_input`.
- Bad: moving staging into the HLS playlist background task when the public
  route currently needs staging failures to surface before the task is spawned.
- Bad: fire-and-forget release that hides release failures after successful
  runner work.

### 6. Tests Required

- App test: local FFmpeg input scope uses the local path without creating an
  `FfmpegInput` staging manifest record.
- App test: remote scoped operation success releases the staged input lease.
- App test: remote scoped operation error releases the staged input lease and
  preserves the operation error.
- App test: release failure after operation success is returned.
- App test: release failure after operation error preserves the operation error.
- Flow tests: HLS and Remux remote staged input release tests continue to pass
  for runner success and runner error.

### 7. Wrong vs Correct

#### Wrong

```rust
let input = app.input.source_input_for_ffmpeg(&source, &uri, &backend).await?;
let result = app.remux.run(input.path.clone()).await;
match result {
    Ok(output) => {
        app.input.release_source_input(input).await?;
        Ok(output)
    }
    Err(err) => {
        let _ = app.input.release_source_input(input).await;
        Err(err)
    }
}
```

This spreads the staging lease invariant across every playback flow and makes
future early returns easy to leak.

#### Correct

```rust
let input = app.input.source_input_scope(&source, &uri, &backend).await?;
app.input
    .with_prepared_source_input(input, |input_path| async move {
        app.remux.run(input_path).await
    })
    .await
```

The flow only receives a path inside the scoped operation; staging, leasing,
release ordering, and release-error priority stay local to `FfmpegInputService`.

## Scenario: Playback Remote Stream Admission

### 1. Scope / Trigger

- Trigger: changing remote Direct Play, remote playback storage streams, or
  playback resource pressure behavior in `nako-server`.

### 2. Signatures

- `LibraryStorageBackend::try_acquire_stream_permit() ->
  Result<OwnedSemaphorePermit>` is the non-blocking stream budget entry point.
- `PlaybackAppService::plan_direct_play(...)` attaches that permit to
  `DirectPlayStreamBody` so the permit lives until the HTTP body is dropped.

### 3. Contracts

- Local Direct Play must not acquire a remote stream permit.
- Remote Direct Play must remain Direct Play first; resource pressure must not
  silently fall back to Remux or HLS.
- The HTTP handler only streams the app service output. It must not implement
  ad hoc semaphore waiting or fallback selection.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Remote stream permit available | `200`/`206` stream response; permit held by body |
| Remote stream budget exhausted | `NakoError::Conflict`, HTTP `409`, code `conflict` |
| Remote stream semaphore closed | storage budget closed error |
| Local source | no remote stream admission |

### 5. Good / Base / Bad Cases

- Good: use `try_acquire_stream_permit()` before opening the remote stream and
  move the permit into `DirectPlayStreamBody`.
- Base: a local file response uses `stream_local_file_response` without remote
  admission.
- Bad: `await` a stream semaphore inside Direct Play or an HTTP handler until
  capacity frees up.

### 6. Tests Required

- App test: remote Direct Play holds the stream permit until the body/plan is
  dropped.
- HTTP test: a second remote Direct Play request under a one-permit budget
  returns `409` with redaction-safe `conflict` evidence.
- Happy-path tests: Direct Play, Remux, and HLS route tests continue to pass.

### 7. Wrong vs Correct

#### Wrong

```rust
let permit = backend.acquire_stream_permit().await?;
```

This hides playback pressure behind an unbounded wait.

#### Correct

```rust
let permit = backend.try_acquire_stream_permit()?;
```

This keeps admission bounded and returns a stable client-safe pressure result.

## Scenario: Library Scan Staging Pressure Admission

### 1. Scope / Trigger

- Trigger: changing library scan start, queued scan scheduling, remote probe
  staging pressure, or storage admission behavior in `nako-server`.

### 2. Signatures

- `StorageBackendRegistry::library_scan_admission_error(&Library) ->
  Result<Option<NakoError>>` is the typed scan-entry and queued-candidate
  admission seam. It composes durable backend health admission with scoped
  staging-pressure admission.
- `JobLeaseRepository::list_claimable_jobs_for_lease(filter, page) ->
  Result<Vec<Job>>` previews queued candidates in the same aged-fairness /
  priority / FIFO order used by durable lease claiming.
- `DurableJobRuntime::claim_next_job_lease(JobLeaseClaimFilter { job_id:
  Some(...), .. }) -> Result<Option<LeasedJob>>` is the exact-claim seam after
  a candidate passes admission.
- `storage_staging_pressure_status(max_bytes, used_bytes) ->
  StorageStagingPressureStatus` is shared by scan admission and Admin
  diagnostics.
- `StorageDiagnosticsAppService::summarize_staging_budget_policy() ->
  Result<Vec<StagingBudgetPolicySlice>>` is the typed per-backend / uniquely
  attributable per-library staging pressure diagnostic boundary.

### 3. Contracts

- Durable `Storage Circuit Breaker` admission runs before staging pressure
  admission.
- Synchronous scan staging admission only blocks libraries that need remote
  probe staging.
- Synchronous remote scan staging admission uses the matching staging budget
  policy slice instead of the global manifest total. When manifest records are
  uniquely attributable to the configured library root, use the library slice;
  otherwise fall back to the backend-scheme slice rather than inventing an
  unsafe per-library attribution.
- Queued background scan scheduling must inspect claimable candidates in durable
  queue order, evaluate per-library storage admission, skip blocked candidates,
  and claim only a runnable candidate by exact job ID.
- Queued scheduling must not claim-and-fail a blocked library scan only because
  another queued library is healthy and runnable.
- If every currently claimable scan candidate is blocked by storage or staging
  pressure, the scheduler returns `BudgetSaturated` and leaves those jobs
  queued.
- Rejection uses `NakoError::storage_staging_budget_exhausted` with a
  redaction-safe synthetic URI, not a Source Locator, local path, credential, or
  backend URL.
- Admin staging diagnostics keep their DTO shape and map from the shared
  classifier. New policy-slice diagnostics must not expose raw Source Locators,
  local paths, fingerprints, credentials, backend URLs, or raw backend errors.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Staging disabled | Scan admission does not block on staging pressure |
| Healthy or Elevated pressure | Scan admission proceeds |
| Critical or Exhausted pressure for the matching remote library/backend staging slice | Synchronous scan fails before scan/probe work starts |
| Critical pressure from an unrelated backend slice | Synchronous remote scan admission proceeds |
| Critical or Exhausted pressure for one remote queued scan while another queued scan is runnable | Blocked remote job stays queued; scheduler continues to the runnable candidate |
| Critical or Exhausted pressure for all currently claimable remote queued scans | Scheduler returns `BudgetSaturated` and leaves jobs queued |
| Local synchronous or queued scan under remote staging pressure | Proceeds because local probe does not require remote staging |

### 5. Good / Base / Bad Cases

- Good: compose scoped staging pressure into `library_scan_admission_error`
  after durable backend health admission, then claim the selected queued job by
  exact ID.
- Base: Admin staging diagnostics call the same pressure classifier used by
  scan admission and expose policy slices from redaction-safe manifest facts.
- Bad: claim the first queued scan job and fail it immediately after discovering
  storage admission would have blocked it, or stop scheduling after the first
  blocked candidate without checking later runnable candidates.
- Bad: attribute same-root multi-endpoint WebDAV staging records to a specific
  library without persisted attribution evidence.

### 6. Tests Required

- App test: remote synchronous scan rejects critical staging pressure before the
  WebDAV listing/probe pipeline starts.
- App test: WebDAV scan admission ignores critical local staging pressure.
- App/Admin test: policy slice attribution covers local and WebDAV records
  without leaking source locators, local paths, credentials, fingerprints, or raw
  backend errors.
- App test: local synchronous scan remains compatible under remote staging
  pressure.
- App test: queued scan scheduling skips a blocked remote library and schedules
  a runnable healthy/local queued scan without failing the blocked job.
- App test: queued scan scheduling leaves a blocked remote job queued under
  critical staging pressure and schedules it after pressure clears.
- Admin test: existing staging pressure threshold mapping continues to pass.

### 7. Wrong vs Correct

#### Wrong

```rust
let leased = runtime.claim_next_job_lease(filter).await?;
// Run then fail only after discovering library admission would block.
```

This drains queued scan jobs and hides runnable later work behind a blocked
candidate.

#### Correct

```rust
let candidates = store
    .list_claimable_jobs_for_lease(filter, PageRequest::new(500, 0))
    .await?;
for candidate in candidates {
    if storage_backends
        .library_scan_admission_error(&library_for(candidate)?)
        .await?
        .is_some()
    {
        continue;
    }

    return runtime
        .claim_next_job_lease(JobLeaseClaimFilter {
            job_id: Some(candidate.id),
            ..filter
        })
        .await;
}
```

This preserves durable queue state for blocked libraries while allowing later
runnable candidates to proceed.

## Scenario: Watch-Folder Runtime Productization

### 1. Scope / Trigger

- Trigger: turning watch-folder discovery, debounce, stable-candidate evidence,
  or incremental library intake into background server behavior.
- Apply this before adding local filesystem watcher loops, polling runtimes, or
  scheduled reconciliation paths that may enqueue library scans.

### 2. Signatures

- `WatchFolderRuntimeAppService::start_enabled_watchers(&RuntimeSupervisor) ->
  Result<WatchFolderRuntimeCoverageReport>` is the startup hook for supervised
  watch-folder runtimes and the coverage diagnostic authority.
- `WatchFolderRuntimeAppService::tick_library(LibraryId) ->
  Result<WatchFolderRuntimeTickDiagnostic>` is the bounded per-library polling
  unit used by tests and runtime loops.
- `AcquisitionIntakeAppService::discover_watch_folder_candidates(...) ->
  Result<WatchFolderDiscoveryDiagnostic>` remains the stable-candidate
  observation authority.
- `WatchFolderSuppressionAppService::begin_planned_write_suppression(...) ->
  Result<PlannedWatchFolderWriteSuppressionDiagnostic>` brackets
  Nako-owned writes that may be visible to watch-folder discovery.
- `WatchFolderSuppressionAppService::complete_planned_write_suppression(token)
  -> Result<Option<CompletePlannedWatchFolderWriteSuppressionDiagnostic>>`
  removes the bracket and reports whether completion requested reconciliation.
- `LibraryScanAppService::admit_watch_folder_library_scan(LibraryId) ->
  Result<LibraryScanAdmissionOutcome>` is the only scan handoff used after
  candidates become newly ready. It may enqueue a new scan or reuse an existing
  queued/running same-library scan.
- `WatchFolderRuntimeTickDiagnostic.scan_admission_status:
  WatchFolderScanAdmissionStatus` is the internal redaction-safe admission
  result reported by a tick: `NotAdmitted` (`not_admitted`), `Enqueued`
  (`enqueued`), `ReusedQueued` (`reused_queued`), or `ReusedRunning`
  (`reused_running`).

### 3. Contracts

- The runtime belongs under `crates/nako-server/src/app/*runtime*` and must be
  spawned through `RuntimeSupervisor`, never through a hidden raw
  `tokio::spawn`.
- Start runtimes only for persisted libraries whose
  `library.options.scan.realtime_monitor` is true and whose first root is a
  local `StorageUri`.
- Startup must preserve redaction-safe watch-folder runtime coverage
  diagnostics for started, disabled, unsupported-root, and missing-root
  libraries instead of only returning a started count.
- Configured-library reconciliation must preserve persisted scan options so
  operator-controlled `realtime_monitor` and scan depth settings survive config
  replay.
- Watch-folder candidate identity must be stable for a locator. Observation
  keys may include size, modified time, etag, or fingerprint evidence, but those
  facts must not be folded into the candidate identity key for new candidates.
- A runtime tick may enqueue a scan only when
  `newly_ready_candidates > 0`; it must use the existing watch-folder scan
  admission path and not execute scan/probe work inline.
- Watch-folder scan admission must coalesce with an existing queued or running
  `JobKind::LibraryScan` for the same Media Library. This coalescing belongs to
  the watch-folder admission path and must not change explicit Admin/manual scan
  commands that intentionally create a new scan job per request.
- Admin-triggered watch-folder discovery must project the same pure
  `nako_library::plan_watch_folder_intake` enqueue decision as runtime ticks.
  Do not derive a separate HTTP-only skip priority from raw candidate counts.
- Planned-write suppression is process-local and TTL-bounded. A suppression
  request must include library ID, `StorageUri` scope, safe owner, safe reason,
  TTL, and completion behavior. Owner/reason are stable identifiers, not raw
  paths or user text.
- Suppression matching is `StorageUri` scheme/path-scope based. A scope matches
  the exact URI and descendants. It must not use host filesystem paths or expose
  raw source locators.
- Suppressed watch-folder entries must not update intake candidates, advance
  stable observation evidence, or enqueue library scan jobs. Completion may
  report reconciliation intent, but broad degraded watcher state is a separate
  follow-on unless explicitly scoped.
- Diagnostics may include library ID, job ID, counts, resource class, and
  redacted refs. `scan_admission_status` may distinguish no admission, newly
  enqueued scan, reused queued scan, and reused running scan, but it must not
  expose raw local paths, Source Locators, fingerprints, etags, credentials, or
  backend URLs.
- `WatchFolderRuntimeAppService` may keep a process-local latest tick cache
  keyed by `LibraryId`. Admin overview may merge that cache into startup
  coverage diagnostics, but the cache must stay inside the runtime helper and
  must only feed redaction-safe summary fields.
- Runtime-loop `Err` logging must convert `NakoError` into a typed safe summary
  before logging. Do not log `%err` directly from watcher ticks; storage and
  provider errors can carry raw URIs, paths, backend URLs, credentials, or raw
  provider text.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| `realtime_monitor` false | No runtime is started; `tick_library` reports `monitored = false`. |
| Library root is non-local or unparsable | No runtime is started; remote watch reliability is not assumed. |
| First supported media observation | Candidate is recorded as `Inspecting`; no scan job is enqueued and `scan_admission_status = NotAdmitted`. |
| Repeated identical supported media observation | Candidate becomes `Ready`; the runtime enqueues one library scan job through `admit_watch_folder_library_scan` and reports `scan_admission_status = Enqueued`. |
| Repeated identical supported media observation while the same library already has a queued scan | Candidate becomes `Ready`; the runtime reuses that queued scan, creates no duplicate job, and reports `scan_admission_status = ReusedQueued`. |
| Repeated identical supported media observation while the same library already has a running scan | Candidate becomes `Ready`; the runtime reuses that running scan, creates no duplicate job, and reports `scan_admission_status = ReusedRunning`. |
| Observation key changes | Stable evidence resets to inspecting before any scan handoff and reports `scan_admission_status = NotAdmitted`. |
| URI is inside active planned-write suppression scope | Discovery increments `suppressed_candidates`, records no candidate, runtime tick enqueues no scan for that URI, and reports `scan_admission_status = NotAdmitted`. |
| Suppression owner/reason is empty, too long, or not a safe identifier | Begin request fails with `NakoError::InvalidInput`. |
| Suppression TTL is zero, negative, or above the configured maximum | Begin request fails with `NakoError::InvalidInput`. |
| Suppression completion uses `ReconcileScope` | Completion removes suppression and reports `reconciliation_requested = true`; the caller decides the supervised reconciliation handoff. |
| Watch-folder discovery/storage error | Tick returns/logs a redaction-safe failure and backs off without bypassing supervision. |
| Fatal runtime tick error returns `Err(NakoError)` | Runtime logs only a safe error class/summary and backs off; raw `NakoError` text is not emitted. |

### 5. Good / Base / Bad Cases

- Good: startup builds one `watch_folder_runtime` task per eligible local
  realtime library, records stable-candidate diagnostics, and enqueues a
  `disk.scan` job only after the second unchanged observation.
- Base: an admin-triggered watch-folder discovery updates intake candidates,
  returns inspecting/ready/newly-ready counts plus the planner's
  `enqueue_scan`/`enqueue_reason`, and does not mutate library sources.
- Base: a Nako-owned NFO/artwork/import write begins a suppression for the
  target `StorageUri`, lets discovery skip that exact URI/descendants, then
  completes the suppression with optional reconciliation intent.
- Bad: a runtime directly scans directories and probes media after a filesystem
  event, or creates another scan executor instead of calling
  `admit_watch_folder_library_scan`.
- Bad: using `size`, fingerprint, etag, or modified time as part of the new
  candidate `source_key`, which prevents repeated observations from updating
  the same candidate.
- Bad: using a host path string, display name, Source Locator, etag,
  fingerprint, or raw error text as suppression owner/reason or Admin
  diagnostic output.
- Bad: logging `Err(err)` from a watch-folder runtime tick with `error = %err`;
  map it to a redaction-safe class/summary first.

### 6. Tests Required

- App test: supervised watch-folder runtime starts for a persisted realtime
  local library and stops when `NakoApp::shutdown_runtime()` is called.
- App/API/HTTP test: watch-folder runtime coverage diagnostics expose started,
  skipped, and latest tick summary status with redacted root references in
  Admin overview.
- App test: first tick records inspecting candidates and enqueues no scan job.
- App test: second identical tick reports newly ready candidates and enqueues a
  `JobKind::LibraryScan` job with resource class `disk.scan`.
- App test: second identical tick with no existing incomplete scan reports
  `scan_admission_status = Enqueued`.
- App test: second identical tick with an existing queued same-library scan
  reports the admitted scan job, `scan_admission_status = ReusedQueued`, and
  does not enqueue a duplicate.
- App test: second identical tick with an existing running same-library scan
  reports the admitted scan job, `scan_admission_status = ReusedRunning`, and
  does not enqueue a duplicate.
- App test: first observations, changed observations, suppressed entries,
  discovery failures, and unmonitored libraries report
  `scan_admission_status = NotAdmitted`.
- Intake/service test: duplicate discovery updates the same candidate and keeps
  supported media in `Inspecting` until the stable observation threshold is
  reached.
- HTTP/Admin test: watch-folder discovery response exposes
  `inspecting_candidates`, `newly_ready_candidates`, `suppressed_candidates`,
  `enqueue_scan`, `enqueue_reason`, and active suppression summaries while
  redacting raw root, source, scope, and token details.
- App test: planned-write suppression matches exact and descendant
  `StorageUri` scopes but not sibling prefixes.
- App test: repeated runtime ticks over a suppressed media file do not enqueue a
  `JobKind::LibraryScan`.
- Unit/app test: watch-folder runtime failure logging maps storage/provider
  errors to redaction-safe summaries and does not expose raw URI, path,
  credential, etag, fingerprint, or backend error text.
- Cross-crate check: `cargo check -p nako-api -p nako-server --tests`.

### 7. Wrong vs Correct

#### Wrong

```rust
tokio::spawn(async move {
    scan_directory(root).await?;
    probe_media_now(source).await?;
});
```

This bypasses the control-plane runtime boundary and creates a second scan/probe
executor.

#### Correct

```rust
runtime.spawn("watch_folder_runtime", "disk.scan.watch_folder", async move {
    match service.tick_library(library_id).await {
        Ok(diagnostic) if diagnostic.newly_ready_candidates > 0 => {
            info!(library_id = %library_id, "watch-folder runtime queued scan");
        }
        Ok(_) => {}
        Err(err) => {
            let safe_error = watch_folder_runtime_safe_error_message(&err);
            warn!(library_id = %library_id, error = %safe_error, "watch-folder tick failed");
        }
    }
});
```

The tick implementation owns the `enqueue_library_scan` call. The runtime loop
keeps the watcher under supervision and lets the existing durable scan queue own
scan execution.

#### Wrong

```rust
Err(err) => warn!(library_id = %library_id, error = %err, "watch-folder tick failed"),
```

This can leak raw storage URIs, paths, backend URLs, credentials, or provider
text carried by `NakoError`.

#### Correct

```rust
Err(err) => {
    let safe_error = watch_folder_runtime_safe_error_message(&err);
    warn!(library_id = %library_id, error = %safe_error, "watch-folder tick failed");
}
```

Runtime tick failure evidence stays useful while preserving the watch-folder
redaction contract.

#### Wrong

```rust
let owner = format!("nfo:{}", raw_local_path.display());
let reason = format!("wrote {source_locator}");
```

This turns host-sensitive strings into diagnostics and makes matching depend on
the wrong authority.

#### Correct

```rust
app.watch_folder_suppression()
    .begin_planned_write_suppression(BeginPlannedWatchFolderWriteSuppressionRequest {
        target_library_id,
        scope_uri,
        owner: "nfo".to_owned(),
        reason: "sidecar_write".to_owned(),
        ttl_ms: Some(60_000),
        completion: PlannedWatchFolderWriteCompletion::ReconcileScope,
    })
    .await?;
```

The suppression request uses `StorageUri` scope and stable safe identifiers.

## Scenario: Playback HLS Lifecycle Orchestration

### 1. Scope / Trigger

- Trigger: changing HLS source startup, HLS playlist startup, transcode session
  reuse, supersede handling, playback resource admission, FFmpeg input staging,
  or playlist readiness waiting in `nako-server`.

### 2. Signatures

- `PlaybackAppService::hls_source_with_policy(...) ->
  Result<HlsSourceOutput>` is a thin app-service entry point.
- `PlaybackAppService::hls_playlist_with_policy(...) ->
  Result<HlsPlaylistOutput>` is a thin app-service entry point.
- `app/playback/hls_flow.rs` owns HLS source context construction, input
  staging, resource admission around playlist start, background HLS execution,
  and playlist readiness waiting.
- `app/playback/hls.rs` owns reserved HLS runner execution and transcode
  session persistence around FFmpeg.
- `PlaybackResourceAdmissionPolicy::HlsStart` is the bounded ordinary HLS
  startup policy. `PlaybackResourceAdmissionPolicy::HlsSupersede` remains the
  bounded replacement policy after older generations are cancelled.

### 3. Contracts

- `nako-playback` remains the pure decision source. Server HLS flow may call the
  planner but must not encode new compatibility rules.
- `nako-transcode` remains the typed pipeline, profile identity, and FFmpeg
  planning source. Server HLS flow must consume typed runtime plans instead of
  building command fragments.
- `PlaybackAppService` should delegate HLS source and playlist lifecycle work to
  `hls_flow`; do not rebuild that lifecycle in broad `mod.rs`.
- HLS artifacts exposed through playlists or segments must stay manifest-driven
  through `hls_artifact` and `playlist` helpers.
- Resource admission must be bounded and must release staged FFmpeg input on
  rejection or runner error.
- Ordinary HLS source and playlist startup must ensure configured capacity and
  acquire the `HlsStart` permit before FFmpeg input staging. If staging fails
  after the permit is acquired, the permit must be released by normal RAII drop.
- HLS supersede must continue to use `HlsSupersede`; do not route supersede
  through `HlsStart`.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Active HLS transcode matches the request key | Wait for that session's playlist readiness |
| Finished HLS transcode matches request key and playlist exists | Reuse completed session |
| Superseded HLS sessions exist | Request cancellation, acquire bounded supersede admission, then start replacement |
| Resource admission rejects playlist startup | Release staged FFmpeg input and return the admission error |
| Running session playlist becomes artifact-ready | Return playlist output without waiting for process exit |
| Finished session playlist is missing | Return `NakoError::storage_io` |
| HLS session is cancelled or failed | Return provider error for `ffmpeg_hls` |
| Playlist readiness timeout expires | Return `NakoError::Conflict` |
| Ordinary HLS start has unconfigured resource capacity | Reject before FFmpeg input staging |
| Ordinary HLS start finds busy resource capacity that releases inside the bounded wait | Start after the permit is released |
| Ordinary HLS start wait expires | Return resource `NakoError::Conflict` without durable queueing |
| FFmpeg input staging fails after HLS start permit acquisition | Return staging error and release the permit by drop |

### 5. Good / Base / Bad Cases

- Good: a public HLS playlist route reaches
  `PlaybackAppService::hls_playlist_with_policy`, which immediately delegates
  to `hls_flow::hls_playlist_with_policy`.
- Base: a direct HLS source request reaches
  `PlaybackAppService::hls_source_with_policy`, which delegates to
  `hls_flow::hls_source_with_policy`.
- Good: ordinary HLS startup calls resource admission with `HlsStart` before
  staging input; supersede calls resource admission with `HlsSupersede`.
- Bad: adding another HLS startup path in `app/playback/mod.rs` that separately
  handles session lookup, supersede, admission, input staging, or playlist
  readiness.
- Bad: moving FFmpeg argv construction or playback compatibility decisions into
  `hls_flow`.
- Bad: staging a remote FFmpeg input before discovering that HLS start capacity
  is configured as unavailable.

### 6. Tests Required

- App tests for HLS source runner start, completed-session reuse, duplicate
  active rejection, supersede, request identity, selected audio/subtitle/HDR
  facts, timeout/failure persistence, and staged input release.
- App tests for HLS playlist running-session readiness, seek supersede,
  cancel-requested permit waiting, `HlsStart` bounded wait, resource pressure
  rejection before input staging, and staged input/permit release on errors.
- HTTP tests for HLS playlist and segment routes, browser ticket protection,
  query-derived audio/subtitle/seek preferences, and running-session playlist
  readiness.
- Gate: `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`
  plus focused `cargo nextest run -p nako-server hls_source --no-fail-fast` and
  `cargo nextest run -p nako-server hls_playlist --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
async fn hls_playlist_with_policy(&self, request: HlsSourceRequest, policy: Option<_>) {
    // session lookup, supersede, admission, input staging, background start,
    // and playlist readiness all live in the broad playback module again.
}
```

This makes the app-service root own HLS lifecycle details and encourages future
HLS feature work to mix server orchestration, playback planning, and transcode
planning.

#### Correct

```rust
async fn hls_playlist_with_policy(&self, request: HlsSourceRequest, policy: Option<_>) {
    hls_flow::hls_playlist_with_policy(self, request, policy).await
}
```

The app-service root remains an entry point, while `hls_flow` owns server-side
HLS lifecycle orchestration and delegates typed planning/execution to the
existing playback and transcode boundaries.

## Scenario: Playback Remux Lifecycle Orchestration

### 1. Scope / Trigger

- Trigger: changing Remux source startup, Remux playback/preflight entry
  points, transcode session reuse, playback resource admission, FFmpeg input
  staging, or Remux output waiting in `nako-server`.

### 2. Signatures

- `PlaybackAppService::remux_source(...) -> Result<RemuxSourceOutput>` is a
  thin app-service entry point.
- `PlaybackAppService::remux_playback_stream(...) ->
  Result<RemuxPlaybackStreamOutput>` is a thin app-service entry point.
- `PlaybackAppService::remux_playback_preflight(...) ->
  Result<RemuxPlaybackPreflightOutput>` is a thin app-service entry point.
- `PlaybackAppService::remux_playback_session_stream(...) ->
  Result<RemuxPlaybackStreamOutput>` is a thin app-service entry point.
- `app/playback/remux_flow.rs` owns Remux source context construction,
  immediate start admission, background start, playback-session linkage,
  FFmpeg input staging/release, session start waiting, and Remux output waiting.
- `app/playback/remux.rs` owns reserved Remux runner execution and transcode
  session persistence around FFmpeg.

### 3. Contracts

- `nako-playback` remains the pure decision source. Server Remux flow may call
  the planner but must not encode new compatibility rules.
- `nako-transcode` remains the typed Remux profile identity and FFmpeg planning
  source. Server Remux flow must consume typed profile/runtime identities.
- `PlaybackAppService` should delegate Remux lifecycle work to `remux_flow`;
  do not rebuild source lookup, start admission, background start, input
  staging, session wait, or output response planning in broad `mod.rs`.
- Remux startup uses immediate playback resource admission. It must not wait,
  durable-queue, or silently fall back to HLS/Direct Play under pressure.
- Active Remux sessions for the same source/request key are reused by playback
  and preflight entry points; completed matching sessions are reused only when
  the persisted output path still exists.
- New Remux playback/preflight sessions must link to the selected transcode
  session before response data is returned.
- Remote staged FFmpeg input must be released after Remux success and after
  runner/admission errors.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Active Remux transcode matches request key | Reuse that session and link the playback session |
| Finished Remux transcode matches request key and output exists | Reuse completed output |
| Remux process permit is busy or unavailable | Return `NakoError::Conflict` immediately |
| Remote FFmpeg input was staged and runner succeeds | Release the staging lease after output is persisted |
| Remote FFmpeg input was staged and runner fails | Release the staging lease and return `ffmpeg_remux` provider error |
| Finished Remux output is missing while serving output | Return storage I/O error |
| Linked playback session points at a non-Remux transcode session | Return invalid input |
| Linked playback session source differs from transcode source | Return invalid input |

### 5. Good / Base / Bad Cases

- Good: Remux HTTP routes call `PlaybackAppService` Remux entry points, which
  immediately delegate to `remux_flow`.
- Base: `remux_flow` may use `remux.rs` for reserved FFmpeg execution and
  persistence; it should not duplicate runner internals.
- Bad: adding another Remux startup path in `app/playback/mod.rs` that handles
  session lookup, admission, staging, background start, or output waiting.
- Bad: making Remux resource pressure wait on the HLS bounded-wait policies.

### 6. Tests Required

- App tests for Remux runner start, completed-session reuse, active-session
  reuse, playback-session linkage, immediate resource-pressure rejection, and
  remote staged input release after success and runner error.
- HTTP tests for Remux GET/range and HEAD/preflight behavior must continue to
  pass without API/DTO changes.
- Gate: `cargo check -p nako-server --tests` plus focused
  `cargo nextest run -p nako-server remux --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
async fn remux_playback_stream(&self, request: RemuxPlaybackStreamRequest) {
    // source lookup, session reuse, admission, staging, background start,
    // playback-session linkage, and response planning all live in mod.rs.
}
```

#### Correct

```rust
async fn remux_playback_stream(&self, request: RemuxPlaybackStreamRequest) {
    remux_flow::remux_playback_stream(self, request).await
}
```

The app-service root remains an entry point, while `remux_flow` owns
server-side Remux lifecycle orchestration and delegates FFmpeg execution to the
existing Remux runner boundary.

## Scenario: Playback Renderer Transport Flow Orchestration

### 1. Scope / Trigger

- Trigger: changing renderer playback session startup, renderer transport
  planning, renderer transport ticket validation at media-route use,
  Direct/Remux/HLS renderer mode selection, renderer playback-session transcode
  linkage, or renderer playback policy enforcement in `nako-server`.

### 2. Signatures

- `PlaybackAppService::start_renderer_playback_session(...) ->
  Result<StartRendererPlaybackSessionOutput>` is a thin app-service entry
  point.
- `app/playback/renderer_flow.rs` owns renderer playback source/probe context,
  effective policy lookup, `RemoteControl` permission enforcement, playback
  planner invocation, mode-specific playback session creation, Remux/HLS
  transcode linkage, and renderer transport plan construction.
- `PlaybackAppService::resolve_renderer_transport_playback_context(...) ->
  Result<ResolvedRendererTransportPlaybackContext>` is the app-service entry
  point for validating a renderer transport ticket when ticketed media routes
  are used.
- `http/renderer.rs` owns renderer command transport ticket and URL authoring.

### 3. Contracts

- `nako-playback` remains the pure decision source. Renderer flow may call the
  planner but must not encode new compatibility rules.
- Direct renderer startup creates a Direct playback session and uses the
  direct plan content type/range facts from the planner decision.
- Remux renderer startup must delegate Remux startup to `remux_flow` and link
  the renderer playback session to the selected Remux transcode session.
- HLS renderer startup must delegate HLS playlist startup to `hls_flow`, link
  the renderer playback session to the selected HLS transcode session, and
  preserve superseded HLS playback-session cancellation.
- Renderer flow returns transport facts only. It must not issue renderer
  tickets, author renderer URLs, or expose raw local paths, locators, command
  lines, playback tickets, or renderer ticket tokens.
- Ticketed playback media routes may parse path/query strings into typed IDs and
  preserve existing optional-query semantics. The app flow owns online renderer
  lookup, renderer transport scope construction, ticket validation, and
  renderer-owner principal matching before returning the principal/session
  context used by Direct, Remux, and HLS playback routes.
- Public renderer route shape, DTOs, generated SDKs, and ticket payloads must
  not change during a flow extraction.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Effective policy denies `RemoteControl` | Return forbidden before starting playback work |
| Planner returns Direct Play | Create Direct playback session and Direct transport plan |
| Planner returns Remux | Start/reuse Remux through `remux_flow`, link playback session, return Remux transport plan |
| Planner returns HLS Transcode | Start/reuse HLS through `hls_flow`, link playback session, cancel superseded HLS playback sessions, return HLS transport plan |
| Planner denies playback | Return the existing playback policy forbidden error |
| Ticketed media route uses a valid renderer transport ticket | Resolve the renderer owner principal and playback session context in app flow |
| Ticket token is blank, expired, mismatched by renderer/session/source/mode/network, or belongs to a different principal than the renderer owner | Return `NakoError::Unauthorized` with message `invalid renderer transport ticket` |

### 5. Good / Base / Bad Cases

- Good: renderer HTTP routes call
  `PlaybackAppService::start_renderer_playback_session`, which immediately
  delegates to `renderer_flow`.
- Good: ticketed playback media routes parse typed route/query IDs, then call
  `PlaybackAppService::resolve_renderer_transport_playback_context` instead of
  constructing `ValidateRendererTransportTicketRequest` in `http/playback.rs`.
- Base: renderer command transport ticket URL construction stays in
  `http/renderer.rs` because it is HTTP/transport mapping, not playback app
  orchestration.
- Bad: duplicating Remux/HLS source lookup, input staging, playlist readiness,
  or FFmpeg runner behavior inside `renderer_flow`.
- Bad: building renderer transport ticket scopes, loading renderer owner state,
  or comparing renderer owner principals inside playback HTTP route handlers.
- Bad: moving renderer ticket issuance or URL authoring into playback app code.

### 6. Tests Required

- HTTP renderer tests for Direct, Remux, and HLS renderer play commands must
  continue to pass without public response shape changes.
- App playback tests must cover successful renderer transport ticket context
  resolution and owner-principal mismatch rejection.
- HTTP renderer transport tests must continue to prove ticketed Direct, Remux,
  HLS playlist, and HLS segment routes preserve status/body behavior.
- Focused playback tests for affected Remux/HLS startup paths should run when
  helper visibility or flow call paths change.
- Gate: `cargo check -p nako-server --tests` plus focused
  `cargo nextest run -p nako-server renderer --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

```rust
async fn start_renderer_playback_session(&self, request: StartRendererPlaybackSessionRequest) {
    // source lookup, policy, planning, Direct/Remux/HLS startup, linkage,
    // and renderer transport plan construction all stay in broad mod.rs.
}
```

This keeps renderer playback as a broad root-module workflow and encourages
future renderer features to mix playback planning, transcode startup, and HTTP
transport concerns.

#### Correct

```rust
async fn start_renderer_playback_session(&self, request: StartRendererPlaybackSessionRequest) {
    renderer_flow::start_renderer_playback_session(self, request).await
}
```

The app-service root remains an entry point, while `renderer_flow` owns
server-side renderer playback orchestration and delegates Remux/HLS details to
their existing focused flow modules.

## Examples

- `http.rs`: central router assembly, auth, network boundary, and API version
  header.
- `http/admin.rs`: Admin route grouping with admin principal enforcement.
- `app/job_runtime.rs`: durable job lease/heartbeat/cancellation boundary.
- `app/playback/resource.rs`: playback runtime resource admission.
- `app/metadata.rs` and `app/metadata_application.rs`: app services wrapping
  metadata workflows.

## Wrong vs Correct

### Wrong

```rust
async fn route_handler(State(app): State<NakoApp>) -> ApiResult<Json<Response>> {
    tokio::spawn(async move {
        // long-running metadata apply work
    });
    Ok(Json(Response::default()))
}
```

### Correct

```rust
async fn route_handler(State(app): State<NakoApp>) -> ApiResult<Json<Response>> {
    let summary = app.metadata_service().apply_review(request).await?;
    Ok(Json(map_summary(summary)))
}
```

Handlers translate and delegate. Durable work goes through the app runtime
boundary.
