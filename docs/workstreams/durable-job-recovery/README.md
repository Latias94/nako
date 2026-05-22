# Durable Job Recovery

Status: Superseded By `durable-job-ownership-leases`
Last updated: 2026-05-17

## 2026-05-19 Supersession Note

`docs/workstreams/durable-job-ownership-leases/` narrows the generic startup
recovery rule. Generic recovery now preserves queued jobs and fails stale
running jobs. Queued rows represent accepted work waiting for a dispatcher or
worker; they are not proof that a previous process abandoned side effects.

## Why This Lane Exists

M38 introduced a durable job runtime helper, but job status persistence still
depends on each app service reaching its own success or failure branch. If a
background job is aborted during shutdown, or if the process exits while a job
is queued or running, the persisted job can remain unfinished forever.

This workstream closes that correctness gap before adding more background job
types to the runtime.

## Relevant Authority

- Related workstreams:
  - `docs/workstreams/server-runtime-deepening/`
  - `docs/workstreams/repository-seam-deepening/`
  - `docs/workstreams/metadata-refresh-seam/`
- Code:
  - `crates/nako-server/src/app/runtime.rs`
  - `crates/nako-server/src/app/startup.rs`
  - `crates/nako-core/src/repository/jobs.rs`
  - `crates/nako-db/src/jobs.rs`

## Scope

- Recover unfinished durable jobs at startup.
- Make startup reports expose recovered job count.
- Add SQLite and server startup regression coverage.
- Remove the unused old `rebuild_search_projection` repository seam if it
  remains uncalled.

## Non-Goals

- No generic durable queue dispatcher.
- No retry policy or resumable job execution.
- No public HTTP API, SDK, or client contract change.
- No new `cancelled` job status unless a later workflow needs it.
- No `CatalogHydrationPort` lookup deepening; that is a separate M42-sized
  architecture task.

## Closeout

M41 shipped startup recovery for stale queued/running durable jobs. The later
ownership-lease lane supersedes that coarse rule: generic startup recovery now
preserves queued jobs and fails stale running jobs.

Remaining architecture work:

- `CatalogHydrationPort` lookup deepening remains a separate M42-sized task.
- A future durable queue dispatcher should revisit whether unfinished jobs are
  failed, retried, or resumed.
