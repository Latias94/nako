# Worker Job Cancellation Checkpoints - Handoff

Status: Active
Last updated: 2026-05-19

## Current State

The lane is active. It follows `durable-job-ownership-leases`, which added
durable job leases, heartbeats, redacted Admin cancel requests, and fenced
`cancel_leased_job`, but deliberately split worker-side cancellation
checkpoints.

`WJCC-020` is complete. `DurableJobRuntime` now supports a context-aware run
path, heartbeat publication of observed cancel intent, cooperative checkpoint
helpers, and fenced acknowledgement of terminal `cancelled`.

## Active Task

- Task ID: `WJCC-030`
- Owner: codex
- Files:
  - `crates/taru-server/src/app/metadata.rs`
- Validation:
  - `cargo nextest run -p taru-server job_cancel --no-fail-fast`
  - `cargo nextest run -p taru-server metadata --no-fail-fast`
- Status: READY
- Review: Metadata maintenance must not emit completed outbox events after a
  cancelled run.
- Evidence: Add focused server tests showing running cancel request becomes
  terminal `cancelled` at a metadata maintenance checkpoint.

## Decisions Since Last Update

- Use cooperative cancellation checkpoints instead of force-killing arbitrary
  worker futures.
- Put the first shared contract in `DurableJobRuntime` because current library
  scan, metadata, and NFO job paths already flow through it.
- Use heartbeat-observed `cancel_requested_at` for the first runtime signal.
- Keep retry/backoff, expired-lease policy, distributed scheduling, and
  process-kill cancellation out of this lane.
- Prefer metadata maintenance as the first real worker integration because it
  naturally loops over item-level side-effect units.
- Existing `run_job` callers intentionally keep the pre-existing success/failure
  interface. Real cooperative cancellation requires migrating a worker to
  `run_job_with_context`.

## Blockers

- None.

## Next Recommended Action

Implement `WJCC-030`: migrate metadata maintenance to
`run_job_with_context`, call `check_cancelled` before each new item refresh, and
ensure cancelled maintenance does not record the metadata-maintenance-completed
event.
