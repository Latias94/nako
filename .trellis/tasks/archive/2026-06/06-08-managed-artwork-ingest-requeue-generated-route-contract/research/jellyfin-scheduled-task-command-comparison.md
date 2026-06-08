# Jellyfin Scheduled Task Command Comparison

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

- Jellyfin exposes elevated Scheduled Task list/detail/start/stop/trigger
  endpoints through a controller that requires elevated policy.
- Jellyfin's start route identifies one task by route ID and delegates to a task
  manager. The controller does not accept arbitrary task payloads from the
  client.
- Jellyfin's task manager queues or executes through scheduled task workers and
  tracks task state outside the controller boundary.
- Nako's Managed Artwork requeue route is closer to Jellyfin's explicit
  operator command shape: one opaque server-owned ID, no raw provider/storage
  material from the caller, and a safe summary response.
- Nako's `artwork/ingests/process-next` route is different: it directly asks the
  server to process the next queued worker item. That is useful for tests/manual
  operation but is not a stable Admin Web command surface yet.

## Decision For This Slice

- Generate `POST /admin/v1/artwork/ingests/{ingest_id}/requeue`.
- Keep `POST /admin/v1/artwork/ingests/process-next` explicitly excluded.
- Do not copy Jellyfin task APIs or task DTOs; use the comparison only to keep
  Nako's operator command boundary explicit.

## Risks To Watch

- Requeue must not expose raw durable job `input_json`, `summary_json`, raw
  error text, provider URLs, storage URIs, local paths, tokens, or artifact
  handles.
- A generated low-level client method must not imply that the read-only
  maintenance page can mutate data without a dedicated live-only workflow task.
- Removing the requeue exclusion must leave the route inventory gate with only
  the intended `process-next` exclusion.
