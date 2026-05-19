# Managed Artwork Artifact Store Drift Inventory

Status: Completed
Last updated: 2026-05-19

## Purpose

This lane splits from `managed-artwork-artifact-lifecycle-cleanup`. Lifecycle
cleanup can now identify and clean unselected Managed Artwork Artifacts. The
remaining storage risk is drift between active database records and local
artifact bytes:

- a DB-backed active artifact can point at bytes that are missing;
- files can remain under the artifact root without any active DB artifact;
- diagnostics must help operators see the drift without exposing local paths or
  internal storage handles.

## Goals

- Add a read-only Admin diagnostics boundary for Managed Artwork Artifact store
  drift.
- Identify DB-backed active artifacts whose expected local file is missing or
  cannot be safely resolved.
- Identify regular files under the artifact root that do not correspond to an
  active DB-backed artifact.
- Return only redacted counts and safe identifiers/media facts.
- Keep repair, deletion, thumbnail variants, ingest runtime controls, and
  gallery/candidate management out of this lane.

## Non-Goals

- Deleting stray files.
- Repairing missing DB-backed artifact files.
- Hashing or validating artifact file contents.
- Thumbnail generation or responsive variants.
- Durable ingest retry, requeue, cancellation, or runtime controls.
- Public gallery/candidate management.
- Public Client API image contract changes.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Current Slice

This lane completed read-only Admin storage drift diagnostics. It does not
delete files, mark rows deleted, expose paths, expose storage URIs, or expose
raw source/cache/provider values.
