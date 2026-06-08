# Jellyfin Scheduled Task Start Comparison

## Reference Files

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ScheduledTasksController.cs`
- `repo-ref/jellyfin/Emby.Server.Implementations/ScheduledTasks/TaskManager.cs`
- `repo-ref/jellyfin/Emby.Server.Implementations/ScheduledTasks/ScheduledTaskWorker.cs`
- Nako comparison points:
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-server/src/app/artwork.rs`
  - `crates/nako-api/src/admin/managed_artwork.rs`
  - `crates/nako-server/src/http/tests/addons.rs`

## Observations

- Jellyfin exposes elevated Scheduled Task list/detail/start/stop endpoints.
- Jellyfin's start route identifies one task by route ID and delegates to the
  task manager. The controller does not accept arbitrary worker payload from
  the client.
- The task manager/worker layer owns execution state and queued execution.
- Nako's `process-next` route is currently already Admin-only and delegates to
  `app.artwork().process_next()`. The HTTP handler does not coordinate storage
  or provider behavior directly.
- Nako's response DTO projects safe state: `processed`, optional ingest,
  optional artifact summary, and optional job summary. Existing API/server tests
  reject storage URI, provider URL/token, cache URI, and local path leaks.

## Decision For This Slice

- Generate `POST /admin/v1/artwork/ingests/process-next`.
- Keep it as a low-level AdminApiClient command only; do not add page controls.
- Remove the last explicit Admin route exclusion so route inventory parity has
  no special cases.
- Do not copy Jellyfin task APIs or task DTOs; use the comparison only to
  validate the controller-to-manager delegation boundary.

## Risks To Watch

- The generated client method must not accept raw provider URL, storage URI,
  job payload, file path, token, or artifact handle input.
- The route still executes one worker step, so UI wiring needs a future
  live-only workflow task instead of piggybacking on the read-only maintenance
  page.
- Empty queue responses must be typed as `processed: false` with nullable
  optional fields, not as an exception or mock success.
