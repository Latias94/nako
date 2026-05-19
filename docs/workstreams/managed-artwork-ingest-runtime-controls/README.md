# Managed Artwork Ingest Runtime Controls

Status: Active
Last updated: 2026-05-19

## Purpose

Managed Artwork ingest can now accept candidates, create durable ingest jobs,
process queued work, store artifacts, and safely fail with redacted summaries.
The missing operational boundary is runtime control: administrators need a
bounded way to retry failed ingest work and later manage cancellation without
exposing raw source URLs, addon payloads, storage handles, paths, or validation
details.

## Goals

- Add explicit Admin controls for retrying failed Managed Artwork ingest work.
- Keep `process-next` as the only fetch/validation/storage executor in the
  first slice.
- Preserve redaction for Admin responses and job summaries.
- Keep artifact publication, cleanup, repair, and public image serving outside
  this lane.
- Document cancellation as a separate state-machine decision unless it becomes
  required for retry safety.

## Non-Goals

- Automatic retry scheduling or background worker orchestration.
- Cancelling an actively fetching in-process ingest in the first slice.
- Adding global `JobStatus::Cancelled` semantics.
- Repairing missing artifacts or re-ingesting deleted artifact bytes.
- Publishing selected artwork after retry success.
- Deleting Managed Artwork Artifacts or artifact files.
- Exposing `storage_uri`, `managed-artwork://...`, local paths, raw source
  URLs, `source_uri`, `cache_uri`, addon payload/provenance JSON, provider query
  strings, addon tokens, raw validation errors, file contents, or content hash
  values.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Current Slice

`MAIRC-020` implements a failed-ingest requeue command:

```text
POST /admin/v1/artwork/ingests/{ingest_id}/requeue
```

The command moves a failed Managed Artwork ingest and its failed durable job
back to queued state, clears safe failure state, does not delete artifacts or
files, and lets the existing `process-next` route retry the same accepted
Artwork Candidate through the normal bounded fetch path.
