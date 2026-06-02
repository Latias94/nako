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
