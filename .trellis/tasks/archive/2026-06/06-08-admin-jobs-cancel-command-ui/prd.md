# Admin Jobs Cancel Command UI

## Problem

Nako already implements `POST /admin/v1/jobs/{job_id}/cancel` and returns the
redaction-safe `AdminJobCancelRequestResponse`, but the route is still excluded
from generated Admin Web route constants and the Jobs page cannot request job
cancellation. Operators can inspect queued/running jobs but cannot perform the
basic control-plane action that the backend already supports.

Jellyfin's Scheduled Tasks API exposes stop/cancel semantics for a running task.
Nako should not copy Jellyfin's scheduled-task framework, but it should connect
the existing durable job cancellation route to the Admin Web Jobs workflow.

## Scope

- Promote `POST /admin/v1/jobs/{job_id}/cancel` into the generated Admin API
  route inventory.
- Regenerate Admin Web TypeScript contracts from `nako-api`.
- Add `AdminApiClient.cancelJob(jobId)` and `AdminDataSource.cancelJob`.
- Add a Jobs page cancel action for queued/running jobs.
- Render mutation feedback from `AdminJobCancelRequestResponse` without
  fabricating terminal state.
- Add Admin Web client, data-source, route, i18n, and redaction tests.

## Non-Goals

- Do not change backend cancellation semantics, job runtime polling, leases, or
  scheduler behavior.
- Do not add a new scheduled-task framework or realtime update channel.
- Do not expose raw job input, summary, error, storage locator, path, token,
  etag, fingerprint, URI digest, or backend payload material.
- Do not add bulk cancellation.

## Acceptance Criteria

- Generated Admin contracts include `NAKO_ADMIN_ROUTES.jobCancel`.
- Admin Web calls the generated cancel route through `AdminApiClient`, not a
  hard-coded string.
- Jobs page shows cancel for queued/running jobs only when the live data source
  supports it.
- Cancel success copy includes the selected job id, returned job status, and
  whether the request was terminal.
- Mock fallback disables cancel actions honestly.
- Existing VFS repair execute/retry actions keep working.
- Focused Admin Web check/test and Admin contract tests pass.
