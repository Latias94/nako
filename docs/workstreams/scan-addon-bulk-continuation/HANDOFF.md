# Scan Addon Bulk Continuation - Handoff

Status: Complete
Last updated: 2026-05-26

## Current State

Nako scan-time Addon bulk metadata scrape now schedules follow-up TaskRuns from
sidecar `next_cursor` output. The scan job still creates only the initial
TaskRun. Continuation happens when an Addon TaskRun completes successfully.

## Shipped Behavior

- Scan payloads include all scanned sources up to the host source limit instead
  of only the first sidecar batch.
- The summary `enqueued_items` reports the bounded payload item count.
- `truncated` reports whether the source query hit the host source limit.
- A successful official-compatible `bulk-metadata-scrape` TaskRun with
  `next_cursor` enqueues another TaskRun.
- The next payload carries `cursor`, plus returned `resume_state`,
  `batch_size`, and `provider_policy` when present.
- Continuations use scheduler/idempotency semantics, not scan-job loops.

## Blockers

None.

## Follow-Ons

- Expose continuation chains in Admin Web task history.
- Add richer task graph/dependency records only if operators need first-class
  lineage beyond idempotency keys.
