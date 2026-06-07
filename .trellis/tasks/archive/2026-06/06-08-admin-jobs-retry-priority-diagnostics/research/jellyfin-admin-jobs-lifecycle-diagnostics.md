# Jellyfin Comparison: Admin Job Lifecycle Diagnostics

## Reference Reviewed

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ScheduledTasksController.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Tasks/TaskInfo.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Tasks/ScheduledTaskHelpers.cs`
- `repo-ref/jellyfin/Jellyfin.Api/WebSocketListeners/ScheduledTasksWebSocketListener.cs`

## Observations

Jellyfin exposes scheduled task state through an elevated API. The task DTO is
not a raw worker object; it projects a bounded operator summary: task identity,
state, progress, trigger list, category, hidden flag, key, and last execution
result. It also has explicit start, stop, and trigger update commands around
that scheduled-task abstraction.

Nako's current baseline differs. ADR 0053 routes durable background work through
persisted jobs and explicit runtime/scheduler seams. Adding a broad Jellyfin-like
scheduled-task catalog before Nako needs it would duplicate concepts and weaken
the durable job boundary.

## Decision

Use Jellyfin as behavior evidence for operator-visible lifecycle state, not as a
source-code template. For this slice, deepen the existing Nako Admin Jobs DTO by
exposing safe durable-job lifecycle facts:

- `priority`
- `attempt`
- `max_attempts`
- `retry_of_job_id`
- `next_attempt_at`

These facts are already stored on the generic `Job` record and do not contain
raw storage identity or raw durable payload material. They are useful for
operators diagnosing retries and queue ordering.

## Redaction Boundary

The new fields do not relax existing redaction rules. Admin Jobs must still
omit raw durable `input_json`, `summary_json`, raw `error`, storage URIs, local
paths, backend URLs, credentials, etags, fingerprints, URI digests, source
locators, provider payloads, and cache payloads.
