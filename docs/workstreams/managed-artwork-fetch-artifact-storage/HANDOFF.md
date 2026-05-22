# Managed Artwork Fetch Artifact Storage Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

MAFA is open. The previous MAIS lane shipped and closed the queued candidate
acceptance boundary. Accepted candidates now create internal
`managed_artwork_ingests` rows and durable `managed_artwork_ingest` jobs.
MAFA-030 now adds the first Nako-owned process-next path: one queued ingest can
be claimed, fetched from an accepted HTTP(S) Artwork Candidate source,
validated as static image content, written to internal local artifact storage,
and committed as `managed_artwork_artifacts` metadata without public artwork
publication. MAFA-040 adds safe failed-job summaries and bounded internal
failure codes, with redaction tests for unsupported media type and invalid
image failures.

## Closed Task

- Task ID: MAFA-050
- Owner: codex
- Files: `docs/workstreams/managed-artwork-fetch-artifact-storage`,
  `docs/api`
- Validation: verify-rust-workstream records fresh final gate evidence
- Status: DONE
- Review: lane closed; public serving, thumbnails, selected artwork
  publication, durable retry/requeue, cancellation, and orphan cleanup are
  follow-on candidates
- Evidence: `EVIDENCE_AND_GATES.md`

## Blockers

- None known.

## Next Recommended Action

- Open a new follow-on before implementing public image behavior. Recommended
  first split: public managed artwork serving and selected artwork publication,
  because clients still have no public image reference for internal artifacts.
- Keep `managed-artwork://...` as an opaque internal storage reference. Do not
  expose raw artifact paths or `storage_uri` in Admin DTOs.
- Keep durable retry/requeue, cancellation, and orphan artifact cleanup as
  separate follow-ons unless the public-serving lane proves they are immediate
  blockers.
