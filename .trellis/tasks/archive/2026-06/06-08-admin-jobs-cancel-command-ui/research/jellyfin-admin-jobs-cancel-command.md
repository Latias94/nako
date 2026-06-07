# Jellyfin Comparison: Stop Task vs Nako Cancel Job

## Reference Studied

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/ScheduledTasksController.cs`

## Findings

Jellyfin exposes `DELETE Running/{taskId}` to stop a running scheduled task. The
controller resolves a scheduled task by id, delegates cancellation to the task
manager, and returns a no-content result. This is paired with Jellyfin's
scheduled-task catalog and task manager lifecycle.

Nako already has a different, durable control-plane primitive:
`POST /admin/v1/jobs/{job_id}/cancel`. It returns typed safe facts through
`AdminJobCancelRequestResponse`, including the redacted job row, whether a
cancel request was recorded, whether the job was terminal, and the cancellation
timestamp.

## Nako Decision

Use the existing durable job cancellation route. Do not introduce a Jellyfin-like
scheduled-task catalog, and do not reinterpret cancel as immediate process stop
inside Admin Web. Admin Web should request cancellation and render the backend
response.

## Redaction Boundary

The UI may render job id, kind, status, resource class, lifecycle counters,
cancel request booleans, and safe timestamps. It must not render durable input
JSON, summary JSON, raw errors, storage locators, local paths, backend URLs,
tokens, etags, fingerprints, URI digests, or cache payloads.
