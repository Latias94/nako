# Durable Job Ownership Leases - Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

The lane is closed. `DJOL-010` recorded the initial design, non-goals, and code
inventory after `job-runtime-worker-control-plane` split durable cancellation
and generic ownership leases into a follow-on.

`DJOL-020` froze the core state-machine vocabulary. `DJOL-030` added the
SQLite schema and repository proof for leases, cancellation request, fenced
completion, and expired-lease recovery. `DJOL-040` wired the shared
`DurableJobRuntime` through exact job claims, heartbeat, and run-token fenced
success/failure. `DJOL-050` added a truthful Admin cancel-request route.

## Closed Task

- Task ID: `DJOL-060`
- Owner: codex
- Files:
  - `docs/workstreams/durable-job-ownership-leases`
- Validation:
  - closeout gate in `EVIDENCE_AND_GATES.md`
- Status: DONE
- Review: lane closed with follow-ons split
- Evidence: `DJOL-020` core contract passed `cargo check -p nako-core --tests`,
  `cargo fmt --all -- --check`, WORKSTREAM JSON parse, and `git diff --check`.
  `DJOL-030` passed DB lease/cancel/startup tests, server startup recovery
  regression, cross-crate check, fmt check, JSON parse, and diff check.
  `DJOL-040` passed runtime leased execution tests, DB job lease tests,
  cross-crate check, and fmt check. `DJOL-050` passed API DTO redaction,
  server cancel-route behavior, and API/server check gates.

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
- `POST /admin/v1/jobs/{job_id}/cancel` is now Admin-only and redacted.
- Queued jobs cancel immediately; running jobs only record cancel intent;
  terminal jobs reject cancellation.
- Worker-side cancellation acknowledgement checkpoints are not implemented in
  this route slice.
- Keep retry/backoff separate from ownership and lease correctness.

## Blockers

- None.

## Follow-Ons

- Worker-side cancellation checkpoints for long-running typed executors.
- Broader worker migrations for metadata, webhook, NFO, automation, scan, and
  probe work that should drain queued jobs without immediate HTTP callers.
- Generic retry/backoff policy over the durable lease model.
- Distributed scheduling or multi-process lease-stealing policy.

## Next Recommended Action

Open a new follow-on for worker-side cancellation checkpoints before claiming
that running job cancellation can stop side effects. Keep retry/backoff and
multi-process scheduling separate unless they become blockers.
