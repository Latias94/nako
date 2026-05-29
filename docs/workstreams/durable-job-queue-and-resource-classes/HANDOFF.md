# Durable Job Queue And Resource Classes - Handoff

Status: Closed
Last updated: 2026-05-29

## Current State

This lane is closed. It followed the completed durable job lanes:

- `job-runtime-worker-control-plane`;
- `durable-job-ownership-leases`;
- `worker-job-cancellation-checkpoints`;
- `durable-job-recovery`;
- `server-runtime-deepening`.

Those lanes gave Nako persisted jobs, startup recovery, ownership leases,
heartbeats, cancellation requests, and worker-side cancellation checkpoints.
It shipped resource class accounting, explicit durable job class to budget
mapping, the first typed scheduler admission path, persisted retry/backoff rows,
and redacted queue pressure diagnostics. Priority policy and broader scheduler
migration remain follow-on work.

## Completed Task

- Task ID: `DJRC-020`
- Owner: codex
- Files:
  - `crates/nako-server/src/app/runtime.rs`
  - `crates/nako-server/src/app/composition.rs`
- Validation:
  - `cargo nextest run -p nako-server runtime_resource_class --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- Status: DONE
- Evidence: runtime registry unit tests and app composition diagnostics test
  passed under the focused `runtime_resource_class` nextest filter.

## Completed Task

- Task ID: `DJRC-030`
- Owner: codex
- Files:
  - `crates/nako-server/src/app/runtime.rs`
  - `crates/nako-server/src/app/jobs.rs`
  - `crates/nako-server/src/app/metadata.rs`
  - `crates/nako-server/src/app/nfo.rs`
  - `crates/nako-server/src/app/addons/intake.rs`
  - `crates/nako-server/src/app/addons/task_runtime.rs`
- Validation:
  - `cargo nextest run -p nako-server job_resource_class --no-fail-fast`
  - `cargo check -p nako-server --tests`
- Status: DONE
- Evidence: job resource mapping tests, server test compile, and prior
  resource registry diagnostics tests passed.

## Completed Task

- Task ID: `DJRC-040`
- Owner: codex
- Files:
  - `crates/nako-server/src/app/job_runtime.rs`
  - `crates/nako-server/src/app/jobs.rs`
  - `crates/nako-server/src/app/tests/startup.rs`
- Validation:
  - `cargo nextest run -p nako-server job_scheduler --no-fail-fast`
  - `cargo nextest run -p nako-server job_runtime --no-fail-fast`
- Status: DONE
- Evidence: scheduler admission test, job runtime tests, and background scan
  regression tests passed.

## Completed Task

- Task ID: `DJRC-050`
- Owner: codex
- Files:
  - `crates/nako-core`
  - `crates/nako-db`
  - `crates/nako-server`
- Validation:
  - `cargo nextest run -p nako-db job_retry --no-fail-fast`
  - `cargo nextest run -p nako-server queue_pressure --no-fail-fast`
  - `cargo check -p nako-core -p nako-db -p nako-api -p nako-server --tests`
- Status: DONE
- Evidence: DB job retry contract, server queue pressure diagnostics test,
  job lease regression tests, scheduler/runtime tests, managed artwork
  regression tests, cross-crate check, formatting, and whitespace gates passed.

## Completed Task

- Task ID: `DJRC-060`
- Owner: planner
- Files:
  - `docs/workstreams/durable-job-queue-and-resource-classes`
- Validation:
  - `verify-rust-workstream`
  - `review-workstream`
  - `Get-Content docs\workstreams\durable-job-queue-and-resource-classes\WORKSTREAM.json | ConvertFrom-Json`
- Status: DONE
- Evidence: closeout review found no blocking findings; focused nextest,
  cross-crate check, formatting, JSON, and whitespace gates passed.

## Decisions Since Last Update

- Keep the first implementation in `nako-server`. Do not split a new crate
  until multiple real production callers need shared types.
- Do not add an external queue engine in this lane.
- Keep current typed workers and service constructors stable in `DJRC-020`.
- Use a resource class registry to own the process-local semaphore pools first;
  scheduler admission and retry/backoff follow in later tasks.
- Treat initial budget class names as coarser pools than durable
  `job.resource_class` strings. Add explicit mapping in `DJRC-030`.
- `DJRC-020` preserves existing service constructors: scan, metadata, NFO,
  addon, and webhook code still receive `Arc<Semaphore>` clones, but those
  semaphores now come from registry-owned resource classes.
- `DJRC-030` keeps durable job execution typed. The mapper is keyed by
  `JobKind` plus safe resource class values, not by blind prefix matching. New
  addon task jobs use fixed `addon.task`; legacy addon task resource strings
  are accepted only so already-persisted rows can still be scheduled.
- `DJRC-040` proves scheduler admission on library scan only. It intentionally
  does not migrate metadata, NFO, artwork, addon, webhook, or automation
  workers yet.
- Scheduler follow-up uses `RuntimeSupervisor::spawn`, not raw `tokio::spawn`,
  to keep the control-plane accounting visible.
- `DJRC-050` persists durable job retry metadata in the job table, creates
  retries as new queued job rows, rejects cancelled/non-failed jobs as retry
  sources, and exposes queue pressure summaries without raw job payloads,
  errors, provider bodies, local paths, or tokens.

## Blockers

- None.

## Follow-Ons

- Next highest-leverage split:
  `proposed:durable-job-priority-policy-and-scheduler-migration`.
- Later split candidates: distributed scheduling, remote workers, addon process
  lifecycle, child-process cancellation, and broader job-kind scheduler
  migration when those become concrete product priorities.

## Next Recommended Action

Open `proposed:durable-job-priority-policy-and-scheduler-migration` when
playback-affecting or user-triggered work needs to outrank maintenance jobs.
Keep any future migration typed at the executor boundary and preserve the
redaction policy from this lane.
