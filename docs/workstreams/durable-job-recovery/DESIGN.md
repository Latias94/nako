# Durable Job Recovery Design

Status: Superseded By `durable-job-ownership-leases`
Last updated: 2026-05-17

## 2026-05-19 Supersession Note

`docs/workstreams/durable-job-ownership-leases/` revises the generic startup
recovery policy. Generic startup recovery now preserves queued jobs and fails
only running jobs that have no typed recovery path. Queued rows represent
accepted work waiting for a dispatcher or worker; they are not proof that a
previous process abandoned side effects.

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

- Startup marks stale running durable jobs as failed with a clear
  stale-startup error.
- Startup reports include the number of recovered jobs.
- Existing terminal jobs remain unchanged.
- The behavior is covered at the SQLite adapter and server startup workflow
  seams.
- Runtime shutdown remains synchronous and does not pretend that asynchronous
  database writes are reliable after task abort.

## In Scope

- `JobRepository` gains an adapter operation for stale unfinished jobs.
- `SqliteStore` originally implemented the operation against queued/running
  jobs. The ownership-lease follow-on narrows generic recovery to running jobs.
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
| Failing running jobs on startup is safer than leaving stale owners active. | High | Running means a worker claimed side effects; queued does not. | Lease-aware recovery can become more precise after every worker uses run tokens. |
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

- stale running jobs are failed during startup,
- recovery is visible in `ServerStartupReport`,
- SQLite and server startup tests pass,
- the old unused search projection helper is removed or explicitly retained,
- focused and workspace validation gates pass,
- and follow-on work is recorded separately.
