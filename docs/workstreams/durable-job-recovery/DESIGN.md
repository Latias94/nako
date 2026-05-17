# Durable Job Recovery Design

Status: Completed
Last updated: 2026-05-17

## Problem

`RuntimeSupervisor::spawn_job` counts success and failure only when the job
future returns. `RuntimeSupervisor::shutdown` cancels the shutdown token and
then aborts registered tasks. Once a task is aborted, the app service branch
that would call `succeed_job` or `fail_job` may never run.

The current startup workflow recovers stale transcode sessions, but it does not
recover unfinished durable jobs. A library scan, metadata refresh, metadata
maintenance, NFO import/export, or future job can therefore remain `queued` or
`running` after restart even though no executor will resume it.

## Target State

- Startup marks unfinished durable jobs as failed with a clear stale-startup
  error.
- Startup reports include the number of recovered jobs.
- Existing terminal jobs remain unchanged.
- The behavior is covered at the SQLite adapter and server startup workflow
  seams.
- Runtime shutdown remains synchronous and does not pretend that asynchronous
  database writes are reliable after task abort.

## In Scope

- `JobRepository` gains an adapter operation for stale unfinished jobs.
- `SqliteStore` implements the operation against queued/running jobs.
- `ServerStartupWorkflow` calls the operation after migrations.
- Startup tests prove stale jobs do not remain unfinished after restart.
- The obsolete `rebuild_search_projection` public helper is removed if no
  caller exists.

## Out Of Scope

- Retrying recovered jobs.
- Persisting cancellation from the sync `Drop` path.
- Background job leasing, heartbeats, ownership tokens, or distributed runners.
- API schema changes for a new job status.
- Catalog hydration lookup reshaping.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Queued and running jobs are not automatically resumed after restart. | High | Current app services spawn jobs in-process when enqueue is called. | If a future dispatcher resumes jobs, recovery must become lease-aware. |
| Failing unfinished jobs on startup is safer than leaving them unfinished. | High | Existing job status model has only queued/running/succeeded/failed. | A future retry model may need a different terminal status. |
| Startup recovery is more reliable than shutdown-time persistence. | High | `Drop` and `AbortHandle::abort` cannot await SQLite writes. | If the server gains async graceful shutdown, this can be complemented later. |

## Architecture Direction

The deepened seam is not inside `RuntimeSupervisor` yet. The runtime owns
in-process task cancellation and diagnostics; the startup workflow owns
cross-process recovery. This keeps the synchronous shutdown path honest and
puts durable consistency behind the storage adapter where it can be tested
deterministically.

The repository operation should express workflow intent: recover unfinished
jobs from a previous process. It should not expose SQL status details to app
services.

## Closeout Condition

This lane can close when:

- unfinished jobs are failed during startup,
- recovery is visible in `ServerStartupReport`,
- SQLite and server startup tests pass,
- the old unused search projection helper is removed or explicitly retained,
- focused and workspace validation gates pass,
- and follow-on work is recorded separately.
