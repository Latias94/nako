# Scan Addon Bulk Continuation - Milestones

Status: Complete
Last updated: 2026-05-26

## SABC-M1 - Red Behavior Captured

`scan_library_continues_addon_bulk_metadata_scrape_from_next_cursor` covered a
13-item library where the official-compatible sidecar batch size is 12. Before
the implementation, Nako reported/enqueued only 12 items and never dispatched a
cursor-12 continuation.

## SABC-M2 - Scheduler-Based Continuation

Nako now enqueues a second Addon TaskRun after the first run succeeds and
returns `next_cursor`. The continuation is not a retry and keeps the library id.

## SABC-M3 - Boundary Cleanup

`task_runtime` only triggers a scan metadata continuation hook. The
official-task-specific payload and idempotency rules live in `scan_metadata`.
