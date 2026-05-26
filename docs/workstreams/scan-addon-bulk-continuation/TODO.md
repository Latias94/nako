# Scan Addon Bulk Continuation - TODO

Status: Complete
Last updated: 2026-05-26

## Tasks

- [x] SABC-010 - Open continuation workstream docs.
- [x] SABC-020 - Add characterization coverage for a scan payload that exceeds
  one sidecar batch.
- [x] SABC-030 - Let scan payloads contain all bounded scan sources instead of
  pre-truncating to one batch.
- [x] SABC-040 - Enqueue continuation TaskRuns from `next_cursor` through the
  Addon task scheduler path.
- [x] SABC-050 - Preserve `resume_state` / sidecar scheduling facts in the next
  payload.
- [x] SABC-060 - Verify focused scan Addon tests and formatting gates.

## Follow-Ups

- Add operator-facing task history UI once the Admin Web lane reaches Addon job
  detail views.
- Revisit explicit task dependency links if Addon TaskRun graphs need to show
  continuation chains beyond idempotency keys.
