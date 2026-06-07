# Jellyfin scheduled tasks comparison

## Reference Files

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ScheduledTasksController.cs`
- `repo-ref/jellyfin/Jellyfin.Api/Controllers/PluginsController.cs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-api/src/extension.rs`

## Jellyfin Behavior Observed

Jellyfin exposes an elevated scheduled task controller with:

- List scheduled tasks with `isHidden` and `isEnabled` filters.
- Get task detail by task ID.
- Start a task by ID.
- Stop/cancel a running task by ID.
- Update a task's trigger list.

The controller is generic: it addresses server scheduled tasks through an
internal task manager. Plugin routes are separate and cover plugin information
and configuration rather than plugin-owned task runs.

## Nako Mapping

Nako's current architecture uses **Addon Task** and **Addon Sidecar** language.
The implemented route family is scoped under one addon:

- `GET /admin/v1/addons/{addon_id}/task-runs`
- `POST /admin/v1/addons/{addon_id}/task-runs`
- `GET /admin/v1/addons/{addon_id}/task-runs/{job_id}`
- `POST /admin/v1/addons/{addon_id}/task-runs/{job_id}/retry`

The DTO model includes redaction-oriented booleans and safe fields:

- `has_input` instead of raw input.
- `safe_error_code` instead of raw sidecar or durable job error.
- `progress` and `result` may contain arbitrary JSON and should not be rendered
  in this Admin Web slice.

## Recommended Slice

Generate Admin route constants for the existing Nako Addon Task Run routes and
surface a bounded operator panel in Admin Web:

- List recent runs for the selected addon.
- Show safe task-run lifecycle facts.
- Retry only retryable failed runs with confirmation.
- Leave creation, cancellation, trigger editing, and generic scheduled task
  management to future tasks.

## Risks

- The generated route inventory must stay synchronized with Axum route strings.
- Frontend rendering must not leak raw Addon Sidecar URLs, credentials, task
  input, task progress/result payloads, or local paths.
- Mock fallback must not pretend a retry mutation succeeded.
