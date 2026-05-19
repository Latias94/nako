# Durable Job Recovery Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M41 is complete and later superseded by `durable-job-ownership-leases`.
Startup recovery originally failed stale queued/running durable jobs because
that covered process crash and abort paths without relying on async persistence
from synchronous shutdown. The ownership-lease follow-on narrows generic
recovery to running jobs and preserves queued jobs for a dispatcher or worker.

## Shipped

- `SqliteStore` can fail unfinished running jobs through `JobRepository`.
- `ServerStartupWorkflow` records recovered job count in
  `ServerStartupReport`.
- SQLite and server startup tests cover the recovery behavior.
- The unused old catalog search projection helper was removed.

## Follow-On

`CatalogHydrationPort` lookup depth remains a separate follow-on. A future
durable queue dispatcher should revisit stale job recovery semantics before it
tries to resume queued/running jobs.
