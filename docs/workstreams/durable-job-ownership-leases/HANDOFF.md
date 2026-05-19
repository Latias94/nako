# Durable Job Ownership Leases - Handoff

Status: Active
Last updated: 2026-05-19

## Current State

The lane is open. `DJOL-010` recorded the initial design, non-goals, and code
inventory after `job-runtime-worker-control-plane` split durable cancellation
and generic ownership leases into a follow-on.

`DJOL-020` froze the core state-machine vocabulary. `DJOL-030` added the
SQLite schema and repository proof for leases, cancellation request, fenced
completion, and expired-lease recovery. `DJOL-040` wired the shared
`DurableJobRuntime` through exact job claims, heartbeat, and run-token fenced
success/failure. No Admin API route has changed in this lane yet.

## Active Task

- Task ID: `DJOL-050`
- Owner: codex
- Files:
  - `crates/taru-api`
  - `crates/taru-server/src/http`
  - `crates/taru-server/src/app/jobs.rs`
  - `docs/api`
- Validation:
  - `cargo nextest run -p taru-server job_cancel --no-fail-fast`
  - `cargo check -p taru-api -p taru-server --tests`
- Status: READY
- Review: Not started
- Evidence: `DJOL-020` core contract passed `cargo check -p taru-core --tests`,
  `cargo fmt --all -- --check`, WORKSTREAM JSON parse, and `git diff --check`.
  `DJOL-030` passed DB lease/cancel/startup tests, server startup recovery
  regression, cross-crate check, fmt check, JSON parse, and diff check.
  `DJOL-040` passed runtime leased execution tests, DB job lease tests,
  cross-crate check, and fmt check.

## Decisions Since Last Update

- Do not add an Admin cancel route before durable ownership exists.
- Treat cancel request as durable intent, not as proof that side effects have
  stopped.
- Add terminal `JobStatus::Cancelled` for acknowledged cancellation.
- Use `JobWorkerId` as diagnostic worker identity.
- Use `JobRunToken` as the write fence for heartbeat, completion, failure, and
  cancellation acknowledgement.
- Add default-unsupported `JobRepository` lease/cancel methods before SQLite
  migration work.
- Add `0029_job_ownership_leases.sql` with worker, run token, heartbeat, lease
  expiry, cancel request, and cancel reason columns.
- Legacy generic startup recovery now preserves queued jobs and fails only
  running jobs without a typed recovery path.
- `JobLeaseClaimFilter` now supports optional `job_id` so a runner can exact
  claim the job it just enqueued.
- `DurableJobRuntime::run_job` now exact-claims, heartbeats, and completes or
  fails with the run-token fence.
- The runtime worker ID is process-local and stable for diagnostics; the run
  token remains the write authority.
- Keep retry/backoff separate from ownership and lease correctness.

## Blockers

- None.

## Next Recommended Action

Run `DJOL-050`: add truthful, redacted Admin cancel-request controls. Keep the
API semantics narrow: queued jobs can become terminal `cancelled`, running jobs
only record cancellation intent until the owning worker acknowledges, and
terminal jobs reject cancel requests.
