# Managed Artwork Fetch Artifact Storage Handoff

Status: Active
Last updated: 2026-05-19

## Current State

MAFA is open. The previous MAIS lane shipped and closed the queued candidate
acceptance boundary. Accepted candidates now create internal
`managed_artwork_ingests` rows and durable `managed_artwork_ingest` jobs, but
no worker fetches remote bytes or writes `managed_artwork_artifacts` yet.

## Active Task

- Task ID: MAFA-030
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-vfs`, `docs/api`,
  `docs/workstreams/managed-artwork-fetch-artifact-storage`
- Validation: focused managed artwork worker tests; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`; `cargo fmt --all -- --check`; `git diff --check`
- Status: READY
- Review: process one queued ingest into internal artifact bytes or a safe
  failure state without public artwork publication
- Evidence: update code/tests/API docs and `EVIDENCE_AND_GATES.md`

## Blockers

- None known.

## Next Recommended Action

- Run MAFA-030. Add a managed-artwork-specific claim/commit repository boundary
  for queued ingest jobs, a local internal artifact storage port/config, a
  bounded HTTP(S) fetcher, an image validation port, and focused success/failure
  tests.
- Use `managed-artwork://...` as an opaque internal storage reference. Do not
  store raw absolute artifact paths as authority in the database.
- Do not create public `ImageAsset` rows, selected artwork, thumbnails, or
  public image references in MAFA-020 or MAFA-030.
- Do not put remote fetch/cache/thumbnailing in the Addon Side Effect handler.
