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
  admission seam.
- `JobLeaseRepository::list_claimable_jobs_for_lease(filter, page) ->
  Result<Vec<Job>>` previews queued candidates in the same aged-fairness /
  priority / FIFO order used by durable lease claiming.
- `DurableJobRuntime::claim_next_job_lease(JobLeaseClaimFilter { job_id:
  Some(...), .. }) -> Result<Option<LeasedJob>>` is the exact-claim seam after
  a candidate passes admission.
- `storage_staging_pressure_status(max_bytes, used_bytes) ->
  StorageStagingPressureStatus` is shared by scan admission and Admin
  diagnostics.

### 3. Contracts

- Durable `Storage Circuit Breaker` admission runs before staging pressure
  admission.
- Synchronous scan staging admission only blocks libraries that need remote
  probe staging.
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
  classifier.

### 4. Validation & Error Matrix

| Condition | Behavior |
|-----------|----------|
| Staging disabled | Scan admission does not block on staging pressure |
| Healthy or Elevated pressure | Scan admission proceeds |
| Critical or Exhausted pressure for remote probe staging | Synchronous scan fails before scan/probe work starts |
| Critical or Exhausted pressure for one remote queued scan while another queued scan is runnable | Blocked remote job stays queued; scheduler continues to the runnable candidate |
| Critical or Exhausted pressure for all currently claimable remote queued scans | Scheduler returns `BudgetSaturated` and leaves jobs queued |
| Local synchronous or queued scan under remote staging pressure | Proceeds because local probe does not require remote staging |

### 5. Good / Base / Bad Cases

- Good: compose staging pressure into `library_scan_admission_error` after
  durable backend health admission, then claim the selected queued job by exact
  ID.
- Base: Admin staging diagnostics call the same pressure classifier used by
  scan admission.
- Bad: claim the first queued scan job and fail it immediately after discovering
  storage admission would have blocked it, or stop scheduling after the first
  blocked candidate without checking later runnable candidates.

### 6. Tests Required

- App test: remote synchronous scan rejects critical staging pressure before the
  WebDAV listing/probe pipeline starts.
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
