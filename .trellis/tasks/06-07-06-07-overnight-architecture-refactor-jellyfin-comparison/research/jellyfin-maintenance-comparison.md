# Jellyfin Maintenance Comparison

## Scope

Read-only comparison only. This file records architecture observations from
`repo-ref/jellyfin` and translates them into Nako-native opportunities without
copying implementation.

## Jellyfin Observations

- Scheduled maintenance work is first-class:
  - `TaskManager` owns registered scheduled task workers.
  - `ScheduledTaskWorker` owns trigger settings and converts trigger metadata
    into concrete daily, weekly, interval, and startup triggers.
  - Maintenance tasks such as cache cleanup, transcode temp cleanup, log cleanup,
    and library refresh are represented as explicit tasks with keys,
    descriptions, progress, and cancellation.
- Cache maintenance is not hidden inside request handlers:
  - old cache and transcode files are cleaned through dedicated scheduled tasks;
  - transcode runtime also performs targeted cleanup for active/failed sessions.
- Library refresh is a scheduled and operator-visible capability rather than
  only an implicit side effect.

## Nako Translation

- Nako already has a stronger durable-job and resource-budget foundation than a
  generic in-process task runner for storage/VFS repair.
- The next useful Nako-native step is not a broad scheduled-task framework. It
  is a non-mutating automatic-policy planner for VFS cache repair that can later
  feed durable job enqueue safely.
- The planner should report:
  - whether automation is enabled;
  - how many unresolved targets are eligible;
  - why targets are blocked;
  - what boundary would be crossed by future automation.
- The planner must not refresh cache, enqueue jobs, purge/delete/invalidate
  cache entries, mutate backend configuration, write library files, or expose raw
  storage identity.

## First Slice

Implement an internal VFS cache repair automation policy planner / dry-run in
`nako-server::app::storage`, with focused tests. Do not add an Admin route until
the internal policy report is stable.
