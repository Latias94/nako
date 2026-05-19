# Managed Artwork Remediation Policy

Status: Completed
Last updated: 2026-05-19

## Purpose

This lane follows `managed-artwork-artifact-store-drift-inventory`. Taru can now
diagnose drift between active Managed Artwork Artifact records and local
artifact-root files. The next boundary is remediation policy: which drift states
can be acted on safely, which must remain operator-visible only, and which must
be split to future repair or re-ingest work.

## Goals

- Provide a redacted Admin remediation plan for Managed Artwork drift.
- Allow explicit cleanup of safe untracked artifact files that have no active
  DB artifact.
- Re-check active DB artifact state before deleting any file.
- Keep missing DB-backed artifacts protected and advisory only.
- Keep Selected Artwork, automatic repair, re-ingest, thumbnails, runtime
  controls, and gallery management out of this lane.

## Non-Goals

- Re-ingesting missing artwork.
- Deleting or unpublishing Selected Artwork.
- Marking active DB-backed artifacts deleted because their files are missing.
- Deleting unrecognized files, unsupported-extension files, or files whose path
  is unexpected for an active artifact.
- Thumbnail generation or responsive variants.
- Durable ingest retry, requeue, cancellation, or runtime controls.
- Public gallery/candidate management.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Current Slice

This lane completed the first remediation boundary: a dry-run remediation plan
and an explicit confirmed command that deletes only cleanable untracked artifact
files. All responses remain redacted.
