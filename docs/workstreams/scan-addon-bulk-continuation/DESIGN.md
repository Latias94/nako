# Scan Addon Bulk Continuation - Design

Status: Implemented
Last updated: 2026-05-26

## Problem

The first scan-time Addon bulk metadata scrape lane only enqueued a bounded first
TaskRun. The official metadata scraper can return `next_cursor` and
`resume_state`, but Nako did not schedule follow-up TaskRuns. That meant a
library scan with more than one sidecar batch silently stopped after the first
batch.

## Boundaries

- Nako owns continuation scheduling.
- Addon Sidecars report progress facts through task output:
  `next_cursor`, optional `resume_state`, optional `batch_size`, and optional
  `provider_policy`.
- The scan job must not run an in-process `while` loop around the sidecar.
- Continuations must use Addon TaskRuns so existing leases, cancellation,
  idempotency, and backpressure continue to apply.

## Shape

`scan_metadata` creates one bounded payload with all scanned sources up to the
host source limit. The sidecar processes from `cursor` for `batch_size` items
and may return a `next_cursor`.

When an Addon TaskRun succeeds, `task_runtime` calls the scan metadata
continuation hook. The hook only recognizes the official-compatible
`bulk-metadata-scrape` task id. If `next_cursor` is forward-moving and still
within the original payload's `items`, it creates another TaskRun with:

- the same declaration id,
- the same dispatch mode,
- the same library/source association,
- an idempotency key suffixed with `:cursor:{next_cursor}`,
- a payload that preserves original items and carries `cursor` plus any returned
  `resume_state`.

Continuation enqueue is best effort after the completed run has already been
persisted. A continuation enqueue failure is logged and does not rewrite the
completed result.

## Non-Goals

- No database schema changes for cursor state in this lane. The Addon TaskRun
  input JSON is the durable scheduling record.
- No new provider breadth or scraper behavior.
- No scan-job loop.
- No retry policy changes beyond existing Addon TaskRun retry behavior.
