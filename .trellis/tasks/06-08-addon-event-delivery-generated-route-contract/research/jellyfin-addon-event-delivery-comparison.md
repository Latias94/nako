# Jellyfin Addon Event Delivery Comparison

## Question

What operator workflow guidance can Nako take from Jellyfin's scheduled-task
and activity-log surfaces for Addon Event Delivery without copying
implementation?

## Findings

- Jellyfin has a broad scheduled-task manager that registers task workers,
  tracks task completion, and exposes task state to operators.
- Jellyfin also has an activity-log model and realtime/session notifications
  for operational visibility.
- Those surfaces are generic. They cover maintenance tasks, library refresh,
  plugin/update work, activity retention, and other system workflows.
- Nako's current Addon Event Delivery model is narrower and more explicit:
  outbox events can be inspected, Addon subscription work can be listed, and an
  operator can deliver or replay Addon delivery for a selected event.
- Nako already has redaction-safe DTOs for this boundary. The missing piece is
  generated route/TypeScript contract visibility and a safe Admin Web route.

## Nako Interpretation

- Do not create a generic scheduled-task UI in this slice.
- Do not add activity-log streaming or realtime notifications in this slice.
- Add a route-owned Events page because the route already has a server-backed
  outbox event list and Addon delivery drilldown commands.
- Keep the projection safe:
  - event ID, kind, status, attempts, payload/error booleans, and timestamps,
  - Addon delivery attempt status/http status/replay reason booleans and safe
    codes,
  - Addon scheduler work status, routing plan status/target, attempt counts,
    and safe reason code,
  - aggregate deliver/replay counts.
- Keep mutations live-only. Mock fallback can show safe rows but must never
  fabricate deliver or replay success.
- Replay must be an explicit operator action with a reason code and a
  confirmation step.

## Files Inspected

- `repo-ref/jellyfin/Emby.Server.Implementations/ScheduledTasks/TaskManager.cs`
- `repo-ref/jellyfin/Emby.Server.Implementations/ScheduledTasks/ScheduledTaskWorker.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Activity/IActivityManager.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Activity/ActivityLogEntry.cs`
- `crates/nako-api/src/extension.rs`
- `crates/nako-server/src/http/addons.rs`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/features/addons/AddonsPage.tsx`
