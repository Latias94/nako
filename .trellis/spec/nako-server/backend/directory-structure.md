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

## Control-Plane Boundary

- Durable jobs, runtime supervision, diagnostics, addon mediation, remote
  access, and API scale contracts are shared control-plane behavior. Check ADR
  0053 before adding hidden per-feature helpers.
- Long-running scan, metadata, playback, addon, webhook, or artifact workflows
  must use durable job/runtime boundaries instead of raw `tokio::spawn`.
- Resource admission belongs in app runtime helpers such as
  `app/playback/resource.rs`, not in pure planner crates.
- Bounded resource-admission policy (for example immediate vs HLS supersede
  wait) should live in the resource helper layer and be reused by orchestration
  code instead of being duplicated in HLS/remux flow modules.

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
- `LibraryScanAppService::enqueue_library_scan(LibraryId) -> Result<Job>` is
  the only scan handoff used after candidates become newly ready.

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
  `newly_ready_candidates > 0`; it must use the existing library scan queue and
  not execute scan/probe work inline.
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
  redacted refs. They must not include raw local paths, Source Locators,
  fingerprints, etags, credentials, or backend URLs.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| `realtime_monitor` false | No runtime is started; `tick_library` reports `monitored = false`. |
| Library root is non-local or unparsable | No runtime is started; remote watch reliability is not assumed. |
| First supported media observation | Candidate is recorded as `Inspecting`; no scan job is enqueued. |
| Repeated identical supported media observation | Candidate becomes `Ready`; the runtime enqueues one library scan job through `enqueue_library_scan`. |
| Observation key changes | Stable evidence resets to inspecting before any scan handoff. |
| URI is inside active planned-write suppression scope | Discovery increments `suppressed_candidates`, records no candidate, and runtime tick enqueues no scan for that URI. |
| Suppression owner/reason is empty, too long, or not a safe identifier | Begin request fails with `NakoError::InvalidInput`. |
| Suppression TTL is zero, negative, or above the configured maximum | Begin request fails with `NakoError::InvalidInput`. |
| Suppression completion uses `ReconcileScope` | Completion removes suppression and reports `reconciliation_requested = true`; the caller decides the supervised reconciliation handoff. |
| Watch-folder discovery/storage error | Tick returns/logs a redaction-safe failure and backs off without bypassing supervision. |

### 5. Good / Base / Bad Cases

- Good: startup builds one `watch_folder_runtime` task per eligible local
  realtime library, records stable-candidate diagnostics, and enqueues a
  `disk.scan` job only after the second unchanged observation.
- Base: an admin-triggered watch-folder discovery updates intake candidates and
  returns inspecting/ready/newly-ready counts without mutating library sources.
- Base: a Nako-owned NFO/artwork/import write begins a suppression for the
  target `StorageUri`, lets discovery skip that exact URI/descendants, then
  completes the suppression with optional reconciliation intent.
- Bad: a runtime directly scans directories and probes media after a filesystem
  event, or creates another scan executor instead of calling
  `enqueue_library_scan`.
- Bad: using `size`, fingerprint, etag, or modified time as part of the new
  candidate `source_key`, which prevents repeated observations from updating
  the same candidate.
- Bad: using a host path string, display name, Source Locator, etag,
  fingerprint, or raw error text as suppression owner/reason or Admin
  diagnostic output.

### 6. Tests Required

- App test: supervised watch-folder runtime starts for a persisted realtime
  local library and stops when `NakoApp::shutdown_runtime()` is called.
- App/API/HTTP test: watch-folder runtime coverage diagnostics expose started
  and skipped status with redacted root references in Admin overview.
- App test: first tick records inspecting candidates and enqueues no scan job.
- App test: second identical tick reports newly ready candidates and enqueues a
  `JobKind::LibraryScan` job with resource class `disk.scan`.
- Intake/service test: duplicate discovery updates the same candidate and keeps
  supported media in `Inspecting` until the stable observation threshold is
  reached.
- HTTP/Admin test: watch-folder discovery response exposes
  `inspecting_candidates`, `newly_ready_candidates`, `suppressed_candidates`,
  and active suppression summaries while redacting raw root, source, scope, and
  token details.
- App test: planned-write suppression matches exact and descendant
  `StorageUri` scopes but not sibling prefixes.
- App test: repeated runtime ticks over a suppressed media file do not enqueue a
  `JobKind::LibraryScan`.
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
        Err(err) => warn!(library_id = %library_id, error = %err, "watch-folder tick failed"),
    }
});
```

The tick implementation owns the `enqueue_library_scan` call. The runtime loop
keeps the watcher under supervision and lets the existing durable scan queue own
scan execution.

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

### 5. Good / Base / Bad Cases

- Good: a public HLS playlist route reaches
  `PlaybackAppService::hls_playlist_with_policy`, which immediately delegates
  to `hls_flow::hls_playlist_with_policy`.
- Base: a direct HLS source request reaches
  `PlaybackAppService::hls_source_with_policy`, which delegates to
  `hls_flow::hls_source_with_policy`.
- Bad: adding another HLS startup path in `app/playback/mod.rs` that separately
  handles session lookup, supersede, admission, input staging, or playlist
  readiness.
- Bad: moving FFmpeg argv construction or playback compatibility decisions into
  `hls_flow`.

### 6. Tests Required

- App tests for HLS source runner start, completed-session reuse, duplicate
  active rejection, supersede, request identity, selected audio/subtitle/HDR
  facts, timeout/failure persistence, and staged input release.
- App tests for HLS playlist running-session readiness, seek supersede,
  cancel-requested permit waiting, resource pressure rejection, and staged input
  release on admission rejection.
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
