# Worker Job Cancellation Checkpoints - Handoff

Status: Active
Last updated: 2026-05-19

## Current State

The lane is active. It follows `durable-job-ownership-leases`, which added
durable job leases, heartbeats, redacted Admin cancel requests, and fenced
`cancel_leased_job`, but deliberately split worker-side cancellation
checkpoints.

`WJCC-020` and `WJCC-030` are complete. `DurableJobRuntime` now supports a
context-aware run path, heartbeat publication of observed cancel intent,
cooperative checkpoint helpers, and fenced acknowledgement of terminal
`cancelled`. Metadata maintenance is the first real worker integration: it
checks cancellation before each item refresh, skips completed outbox publication
for cancelled runs, and lets runtime diagnostics count cancelled jobs
separately from successful jobs.

## Active Task

- Task ID: `WJCC-040`
- Owner: codex
- Files:
  - `crates/taru-server/src/app/jobs.rs`
  - `crates/taru-server/src/app/nfo.rs`
  - `docs/api/HTTP_API.md`
- Validation:
  - `cargo nextest run -p taru-server job_runtime --no-fail-fast`
  - targeted package checks for touched modules
- Status: READY
- Review: Checkpoints must sit before new side-effect units, not after success
  events or after irreversible writes.
- Evidence: Add tests or documented split decisions for library scan/probe and
  NFO import/export boundaries.

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
- Runtime diagnostics now expose `cancelled_jobs` separately from
  `succeeded_jobs`; Admin overview should not report acknowledged cancellation
  as success.
- Use low-concurrency validation commands for this workspace unless the user
  explicitly asks for broader parallel verification:
  `CARGO_BUILD_JOBS=2`, `NEXTEST_TEST_THREADS=1`, and `cargo nextest run -j 2`.

## Blockers

- None.

## Next Recommended Action

Implement `WJCC-040`: audit library scan/probe and NFO import/export jobs,
then either add safe cooperative checkpoints before their next side-effect unit
or split deeper migrations into named follow-on lanes.
