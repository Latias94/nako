# Managed Artwork Public Serving Selection Handoff

Status: Active
Last updated: 2026-05-19

## Current State

MAPS is open. The previous MAFA lane stores validated Managed Artwork bytes as
internal artifacts and keeps `managed-artwork://...` storage authority private.
This lane now owns the public boundary: Selected Artwork publication, redacted
Public Client image references, and first-party image byte serving.

## Current Task

- Task ID: MAPS-020
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-api`,
  `crates/taru-client-protocol`, `crates/taru-server`, `docs/api`,
  `docs/workstreams/managed-artwork-public-serving-selection`
- Validation: audit inventory plus `git diff --check`
- Status: READY
- Review: freeze DTO/route/schema shape before implementation
- Evidence: `EVIDENCE_AND_GATES.md`

## Blockers

- None known.

## Next Recommended Action

- Run MAPS-020 and make the public image contract explicit before adding schema
  migrations or route handlers.
- Prefer a new Selected Artwork model over reusing `ImageAsset.selected`.
- Treat old public `source_uri`, `cache_uri`, `ImageRefDto.uri`, and
  `selected` fields as leak-prone unless proven internal-only.
- Keep thumbnails, durable retry/requeue, cancellation, and orphan artifact
  cleanup split from the first serving path.
