# Jellyfin Comparison: Queue Pressure Diagnostics

## Reference Studied

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ScheduledTasksController.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Tasks/TaskInfo.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Tasks/ScheduledTaskHelpers.cs`
- `repo-ref/jellyfin/Jellyfin.Api/WebSocketListeners/ScheduledTasksWebSocketListener.cs`
- `repo-ref/jellyfin/Emby.Server.Implementations/ScheduledTasks/ScheduledTaskWorker.cs`

## Findings

Jellyfin models scheduled tasks as a first-class operator surface. The API lists
task summaries and task execution state, while WebSocket listeners notify
clients when tasks execute, complete, or report progress. That design is
appropriate for Jellyfin's scheduled-task catalog.

Nako's architecture has a different control-plane baseline. Work is represented
as durable jobs with kind, resource class, status, retry lifecycle, claimability,
and repository-owned pressure aggregation. The queue is already the durable
truth for scan, source hash, VFS repair, automation, and future background
flows.

## Nako Decision

Do not introduce a Jellyfin-like scheduled-task framework in this slice. Expose
the existing durable job queue-pressure read model through the Admin Jobs list
response instead. This gives operators the same class of summary visibility
while preserving Nako's resource-class and durable retry boundaries.

Realtime updates remain a follow-on. If Nako adds realtime Admin Jobs
diagnostics later, it should publish redaction-safe queue-pressure snapshots or
job lifecycle events from the durable queue/runtime boundary, not from a
parallel scheduled-task catalog.

## Redaction Boundary

Queue pressure may expose grouped kind, status, resource class, counts, and
timestamps. It must not expose durable input JSON, summary JSON, raw errors,
storage locators, paths, URI digests, etags, fingerprints, credentials, source
locators, or backend/cache payload material.
