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
publication.

## Active Task

- Task ID: MAFA-040
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-api`, `docs/api`,
  `docs/workstreams/managed-artwork-fetch-artifact-storage`
- Validation: focused failure/redaction tests; `cargo nextest run -p taru-server artwork --no-fail-fast`; `cargo nextest run -p taru-db artwork --no-fail-fast`; `git diff --check`
- Status: READY
- Review: harden retry/cancellation, safe failure codes, job summaries, and
  admin diagnostics without leaking raw URLs, paths, storage URIs, provider
  tokens, or decoder internals
- Evidence: add failure tests and update `EVIDENCE_AND_GATES.md`

## Blockers

- None known.

## Next Recommended Action

- Run MAFA-040. Add explicit tests for unsupported media type, invalid image,
  too-large responses, fetch timeout/status failures, and redacted failure
  responses/job errors.
- Decide whether failed `managed_artwork_ingest` rows should be retryable in
  place, re-queued through a new job, or left terminal until a later retry API.
- Keep `managed-artwork://...` as an opaque internal storage reference. Do not
  expose raw artifact paths or `storage_uri` in Admin DTOs.
- Do not create public `ImageAsset` rows, selected artwork, thumbnails, public
  image references, or Addon Side Effect fetch/cache behavior in MAFA-040.
