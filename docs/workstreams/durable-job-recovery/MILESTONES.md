# Durable Job Recovery Milestones

Status: Completed
Last updated: 2026-05-17

## M41.0 - Workstream Opened

Exit criteria:

- Problem, target state, scope, and non-goals documented.
- Task ledger has independently validatable slices.
- Evidence gates are defined before code changes.

Status: complete.

## M41.1 - Unfinished Job Recovery Port

Exit criteria:

- `JobRepository` exposes a workflow-shaped operation for stale unfinished
  jobs.
- SQLite marks queued/running jobs failed and leaves succeeded/failed jobs
  unchanged.
- Adapter-level regression test passes.

Status: complete.

## M41.2 - Startup Integration

Exit criteria:

- `ServerStartupWorkflow` invokes job recovery after migration.
- `ServerStartupReport` includes recovered job count.
- Server startup regression test proves stale jobs are failed after restart.

Status: complete.

## M41.3 - Old Seam Removal

Exit criteria:

- `rebuild_search_projection` is removed or narrowed.
- No caller relies on the old `CatalogRepository + MediaRepository +
  SearchIndex` projection entrypoint.

Status: complete.

## M41.4 - Closeout

Exit criteria:

- Focused server/db/catalog checks pass.
- Workspace check and nextest pass.
- Roadmap, goal map, workstream index, and handoff reflect shipped behavior.

Status: complete.
