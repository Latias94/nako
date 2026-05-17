# Durable Job Recovery

Status: Completed
Last updated: 2026-05-17

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
  - `crates/taru-server/src/app/runtime.rs`
  - `crates/taru-server/src/app/startup.rs`
  - `crates/taru-core/src/repository/jobs.rs`
  - `crates/taru-db/src/jobs.rs`

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

M41 shipped startup recovery for stale queued/running durable jobs. This closes
the correctness gap for process crash and forced abort paths, where in-process
shutdown cleanup cannot be trusted.

Remaining architecture work:

- `CatalogHydrationPort` lookup deepening remains a separate M42-sized task.
- A future durable queue dispatcher should revisit whether unfinished jobs are
  failed, retried, or resumed.
