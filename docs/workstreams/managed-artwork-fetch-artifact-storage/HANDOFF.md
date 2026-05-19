# Managed Artwork Fetch Artifact Storage Handoff

Status: Active
Last updated: 2026-05-19

## Current State

MAFA is open. The previous MAIS lane shipped and closed the queued candidate
acceptance boundary. Accepted candidates now create internal
`managed_artwork_ingests` rows and durable `managed_artwork_ingest` jobs.
MAFA-030 now adds the first Taru-owned process-next path: one queued ingest can
be claimed, fetched from an accepted HTTP(S) Artwork Candidate source,
validated as static image content, written to internal local artifact storage,
and committed as `managed_artwork_artifacts` metadata without public artwork
publication. MAFA-040 adds safe failed-job summaries and bounded internal
failure codes, with redaction tests for unsupported media type and invalid
image failures.

## Active Task

- Task ID: MAFA-050
- Owner: codex
- Files: `docs/workstreams/managed-artwork-fetch-artifact-storage`,
  `docs/api`
- Validation: verify-rust-workstream records fresh final gate evidence
- Status: READY
- Review: close the fetch/artifact lane or split public serving, thumbnails,
  selected artwork publication, durable retry, and cancellation into narrower
  follow-ons
- Evidence: update closeout notes and final gate evidence

## Blockers

- None known.

## Next Recommended Action

- Run MAFA-050. Review MAFA-030/040 evidence, decide whether this workstream is
  ready to close, and split follow-ons for public image serving, thumbnails,
  selected artwork publication, durable retry/requeue, cancellation, and orphan
  artifact cleanup.
- Keep `managed-artwork://...` as an opaque internal storage reference. Do not
  expose raw artifact paths or `storage_uri` in Admin DTOs.
- Do not create public `ImageAsset` rows, selected artwork, thumbnails, public
  image references, or Addon Side Effect fetch/cache behavior during closeout.
