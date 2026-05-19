# Durable Job Ownership Leases - Handoff

Status: Active
Last updated: 2026-05-19

## Current State

The lane is open. `DJOL-010` recorded the initial design, non-goals, and code
inventory after `job-runtime-worker-control-plane` split durable cancellation
and generic ownership leases into a follow-on.

`DJOL-020` froze the core state-machine vocabulary. `DJOL-030` added the
SQLite schema and repository proof for leases, cancellation request, fenced
completion, and expired-lease recovery. No Admin API route has changed in this
lane yet.

## Active Task

- Task ID: `DJOL-040`
- Owner: codex
- Files:
  - `crates/taru-server/src/app/runtime.rs`
  - `crates/taru-server/src/app/job_runtime.rs`
  - one typed app-service execution path
- Validation:
  - `cargo nextest run -p taru-server job_runtime --no-fail-fast`
  - `cargo nextest run -p taru-server startup --no-fail-fast`
- Status: NEEDS_CONTEXT
- Review: Not started
- Evidence: `DJOL-020` core contract passed `cargo check -p taru-core --tests`,
  `cargo fmt --all -- --check`, WORKSTREAM JSON parse, and `git diff --check`.
  `DJOL-030` passed DB lease/cancel/startup tests, server startup recovery
  regression, cross-crate check, fmt check, JSON parse, and diff check.

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
- Keep retry/backoff separate from ownership and lease correctness.

## Blockers

- None.

## Next Recommended Action

Run `DJOL-040`: wire one real runtime path through the leased repository
contract. Prefer a narrow server-runtime proof before adding Admin cancel
routes.
