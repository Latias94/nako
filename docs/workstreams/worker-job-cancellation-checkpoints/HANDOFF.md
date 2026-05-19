# Worker Job Cancellation Checkpoints - Handoff

Status: Active
Last updated: 2026-05-19

## Current State

The lane is open. It follows `durable-job-ownership-leases`, which added durable
job leases, heartbeats, redacted Admin cancel requests, and fenced
`cancel_leased_job`, but deliberately split worker-side cancellation
checkpoints.

## Active Task

- Task ID: `WJCC-020`
- Owner: codex
- Files:
  - `crates/taru-server/src/app/job_runtime.rs`
- Validation:
  - `cargo nextest run -p taru-server job_runtime --no-fail-fast`
  - `cargo check -p taru-server --tests`
- Status: NEEDS_CONTEXT
- Review: Runtime must distinguish cancellation from failure.
- Evidence: Runtime tests should show terminal `JobStatus::Cancelled`.

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

## Blockers

- None.

## Next Recommended Action

Implement `WJCC-020`: add the runtime cancellation context/checkpoint API,
update the heartbeat loop to publish observed cancel intent, and make
`DurableJobRuntime` persist terminal `cancelled` with `cancel_leased_job` when
the operation reports observed cancellation.
