# Managed Artwork Fetch Artifact Storage Handoff

Status: Active
Last updated: 2026-05-19

## Current State

MAFA is open. The previous MAIS lane shipped and closed the queued candidate
acceptance boundary. Accepted candidates now create internal
`managed_artwork_ingests` rows and durable `managed_artwork_ingest` jobs, but
no worker fetches remote bytes or writes `managed_artwork_artifacts` yet.

## Active Task

- Task ID: MAFA-020
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-vfs`, `docs/workstreams/managed-artwork-fetch-artifact-storage`
- Validation: audit gate plus `git diff --check`
- Status: READY
- Review: decide the first internal artifact byte storage policy and whether a
  new storage port is required before worker implementation
- Evidence: update `EVIDENCE_AND_GATES.md` and `DESIGN.md`

## Blockers

- None known.

## Next Recommended Action

- Run MAFA-020. Audit job runtime claim/update behavior, storage/VFS/cache and
  staging seams, HTTP fetch policy, image validation options, and artifact
  commit requirements.
- Do not process `managed_artwork_ingest` jobs until artifact byte storage and
  redacted failure semantics are explicit.
- Do not create public `ImageAsset` rows, selected artwork, thumbnails, or
  public image references in MAFA-020 or MAFA-030.
- Do not put remote fetch/cache/thumbnailing in the Addon Side Effect handler.
