# Durable Job Ownership Leases - Handoff

Status: Active
Last updated: 2026-05-19

## Current State

The lane is open. `DJOL-010` records the initial design, non-goals, and code
inventory after `job-runtime-worker-control-plane` split durable cancellation
and generic ownership leases into a follow-on.

No schema, API, or runtime behavior has changed in this lane yet.

## Active Task

- Task ID: `DJOL-020`
- Owner: codex
- Files:
  - `docs/adr/0006-persist-job-inputs-and-explicit-retry-policy.md`
  - `crates/taru-core/src/job.rs`
  - `crates/taru-core/src/repository/jobs.rs`
  - `docs/workstreams/durable-job-ownership-leases/DESIGN.md`
  - `docs/workstreams/durable-job-ownership-leases/TODO.md`
- Validation:
  - `cargo check -p taru-core --tests`
  - `cargo fmt --all -- --check`
- Status: NEEDS_CONTEXT
- Review: Not started
- Evidence: Opening inventory only

## Decisions Since Last Update

- Do not add an Admin cancel route before durable ownership exists.
- Treat cancel request as durable intent, not as proof that side effects have
  stopped.
- Prefer a run-token fence in addition to diagnostic worker identity.
- Keep retry/backoff separate from ownership and lease correctness.

## Blockers

- None.

## Next Recommended Action

Run `DJOL-020`: freeze the state machine and repository contract names. Decide
whether Taru should add terminal `JobStatus::Cancelled` or initially encode
acknowledged cancellation as a safe failed outcome before any migration is
written.
