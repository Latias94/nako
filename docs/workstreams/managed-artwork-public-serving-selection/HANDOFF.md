# Managed Artwork Public Serving Selection Handoff

Status: Active
Last updated: 2026-05-19

## Current State

MAPS is open. The previous MAFA lane stores validated Managed Artwork bytes as
internal artifacts and keeps `managed-artwork://...` storage authority private.
This lane now owns the public boundary: Selected Artwork publication, redacted
Public Client image references, and first-party image byte serving. MAPS-020
has frozen the contract:

- `selected_artworks.id` is the public image ID authority.
- `GET/HEAD /images/{image_id}` serves selected image bytes.
- `POST /admin/v1/artwork/artifacts/{artifact_id}/publish` publishes a stored
  artifact as Selected Artwork.
- `PublicImageRefDto` replaces leak-prone Public Client image DTOs.
- `ImageAsset` remains internal/provenance only and is not the public selected
  artwork authority.

## Current Task

- Task ID: MAPS-030
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-api`,
  `crates/taru-server`, `docs/api`
- Validation: focused db publication tests; focused admin HTTP tests;
  `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`;
  `cargo fmt --all -- --check`; `git diff --check`
- Status: READY
- Review: implement Selected Artwork publication before public byte serving
- Evidence: `EVIDENCE_AND_GATES.md`

## Blockers

- None known.

## Next Recommended Action

- Run MAPS-030 by adding `SelectedArtworkId`,
  `0027_selected_artwork_publication.sql`, selected-artwork repository records
  and methods, and the Admin publish command.
- Keep the Admin response redacted: no `storage_uri`, source URL, `cache_uri`,
  local path, or addon/provider token material.
- Do not start MAPS-040 public byte serving until a selected record can be
  created and read.
- Keep thumbnails, durable retry/requeue, cancellation, and orphan artifact
  cleanup split from the first serving path.
