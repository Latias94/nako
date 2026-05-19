# Durable Job Ownership Leases - Handoff

Status: Active
Last updated: 2026-05-19

## Current State

The lane is open. `DJOL-010` recorded the initial design, non-goals, and code
inventory after `job-runtime-worker-control-plane` split durable cancellation
and generic ownership leases into a follow-on.

`DJOL-020` freezes the core state-machine vocabulary. No schema, API route, or
runtime behavior has changed in this lane yet.

## Active Task

- Task ID: `DJOL-030`
- Owner: unassigned
- Files:
  - `crates/taru-db/migrations/`
  - `crates/taru-db/src/jobs.rs`
  - `crates/taru-db/src/codec.rs`
  - `crates/taru-db/src/tests.rs`
- Validation:
  - `cargo nextest run -p taru-db job_lease --no-fail-fast`
  - `cargo nextest run -p taru-db job_cancel --no-fail-fast`
- Status: NEEDS_CONTEXT
- Review: Not started
- Evidence: `DJOL-020` core contract passed `cargo check -p taru-core --tests`,
  `cargo fmt --all -- --check`, WORKSTREAM JSON parse, and `git diff --check`

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
- Keep retry/backoff separate from ownership and lease correctness.

## Blockers

- None.

## Next Recommended Action

Run `DJOL-030`: add SQLite schema columns and adapter tests for claim,
heartbeat, stale-token rejection, fenced success/failure, cancel request,
queued cancellation, cancellation acknowledgement, and expired lease recovery.
