# Addon Task Runtime Contract - Handoff

Status: Completed
Last updated: 2026-05-23

## Current State

Nako now has a host-owned Addon Task runtime surface. Manifest task
declarations still only describe what an Addon Sidecar can do; Nako creates a
separate `AddonTaskRun` backed by `JobKind::AddonTask` when work should be
scheduled. The runtime now supports both sidecar-claimed runs and direct
host-dispatched task-path runs.

Implemented pieces:

- `AddonTaskRun` domain/repository contract in `nako-core`.
- SQLite/PostgreSQL `addon_task_runs` storage keyed by `job_id`.
- Admin routes for create/list/get/retry.
- Addon Token protected runtime routes for claim/progress/complete/fail/cancel
  acknowledgement, with the claim/progress lease carrying the execution input
  for the sidecar.
- Direct dispatch mode that lets Nako claim the specific queued run and call
  the declared sidecar task `path` with a host-owned task envelope.
- Progress/result schemas owned by Nako:
  - `nako.addon.task_run.progress.v1`
  - `nako.addon.task_run.result.v1`
- Focused HTTP tests for success, failure retry, and cancellation
  acknowledgement.
- Focused direct-dispatch tests for task-path success, retry after HTTP
  failure, and in-flight host cancellation.

## Next Task

Open a follow-on lane when one of the remaining product areas is ready.

Recommended follow-ons:

- authenticated outbound task dispatch credential storage and resolution for
  `AddonAuth::Bearer` and `AddonAuth::SharedSecret`;
- official-addon task-path smoke coverage once an official addon exposes a task
  declaration;
- Addon Source Catalog / marketplace discovery;
- package signing and trust-root policy;
- provider breadth beyond the first companion addon;
- process/container supervision, if Nako decides to own sidecar execution.

## Known Risks

- A task runtime lane can accidentally absorb source catalog or marketplace
  discovery if the first slice is not narrow.
- The existing manager-plan and official addon smoke must stay valid while the
  task runtime lane evolves.
- Progress/result semantics may need their own task-state fixtures if the lane
  grows beyond the first execution slice.
- Authenticated outbound sidecar task dispatch still needs credential
  management; this runtime lane should not invent secret storage implicitly.
- The current official addon smoke covers health and resource diagnostics, not
  task-path dispatch.
