# Addon Task Runtime Contract - Handoff

Status: Active
Last updated: 2026-05-23

## Current State

Nako now has a first host-owned Addon Task runtime surface. Manifest task
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

Continue with ATRC-060 closeout or split follow-ons.

Suggested next steps:

1. Run the broader addon gate:
   `cargo nextest run -p nako-server addon --no-fail-fast`.
2. Run `cargo fmt --all -- --check` and `git diff --check`.
3. Decide whether authenticated outbound task dispatch credential storage
   belongs in this lane or a follow-on. The current direct-dispatch tests cover
   `AddonAuth::None`; `Bearer`/`SharedSecret` outbound task dispatch needs a
   real secret source rather than reusing Addon Tokens.
4. Keep source catalog, package signing, provider breadth, and process
   supervision out of this runtime contract.

## Known Risks

- A task runtime lane can accidentally absorb source catalog or marketplace
  discovery if the first slice is not narrow.
- The existing manager-plan and official addon smoke must stay valid while the
  task runtime lane evolves.
- Progress/result semantics may need their own task-state fixtures if the lane
  grows beyond the first execution slice.
- Authenticated outbound sidecar task dispatch still needs credential
  management; this runtime lane should not invent secret storage implicitly.
