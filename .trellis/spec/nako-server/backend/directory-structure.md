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
  Result<Option<NakoError>>` is the typed scan-entry admission seam. It composes
  durable backend health admission with scoped staging-pressure admission.
- `StorageBackendRegistry::queued_library_scan_budget_saturated() ->
  Result<bool>` is the queued scheduler pressure guard.
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
- Queued background scan scheduling may use global staging pressure to avoid
  claiming jobs during critical pressure. Do not add scheduler fairness or
  mixed local/remote queue bypass behavior in the storage policy slice; that is
  a scheduler lane.
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
| Critical or Exhausted global pressure during queued scheduling | Scheduler returns `BudgetSaturated` and leaves jobs queued |
| Local synchronous scan under remote staging pressure | Proceeds because local probe does not require remote staging |

### 5. Good / Base / Bad Cases

- Good: compose scoped staging pressure into `library_scan_admission_error`
  after durable backend health admission.
- Base: Admin staging diagnostics call the same pressure classifier used by
  scan admission and expose policy slices from redaction-safe manifest facts.
- Bad: start a durable queued scan job, then fail it immediately only because
  global staging pressure was already critical.
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
- App test: queued scan scheduling leaves the job queued under critical staging
  pressure and schedules it after pressure clears.
- Admin test: existing staging pressure threshold mapping continues to pass.

### 7. Wrong vs Correct

#### Wrong

```rust
let leased = runtime.claim_next_job_lease(filter).await?;
// Run then fail only after discovering staging pressure.
```

This drains queued scan jobs while pressure is known in advance.

#### Correct

```rust
if storage_backends.queued_library_scan_budget_saturated().await? {
    return Ok(LibraryScanScheduleOutcome::BudgetSaturated);
}
```

This preserves durable queue state until staging pressure is healthy enough to
claim work.

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
