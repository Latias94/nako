# Managed Artwork Public Serving Selection Handoff

Status: Active
Last updated: 2026-05-19

## Current State

MAPS is open. The previous MAFA lane stores validated Managed Artwork bytes as
internal artifacts and keeps `managed-artwork://...` storage authority private.
This lane now owns the public boundary: Selected Artwork publication, redacted
Public Client image references, and first-party image byte serving. MAPS-020
froze the contract and MAPS-030 shipped explicit Admin publication:

- `selected_artworks.id` is the public image ID authority.
- `POST /admin/v1/artwork/artifacts/{artifact_id}/publish` publishes a stored
  artifact as Selected Artwork.
- `PublicImageRefDto` replaces leak-prone Public Client image DTOs.
- `ImageAsset` remains internal/provenance only and is not the public selected
  artwork authority.
- `GET/HEAD /images/{image_id}` and Public Client catalog DTO replacement are
  still MAPS-040 work.

## Current Task

- Task ID: MAPS-040
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-api`,
  `crates/taru-client-protocol`, `crates/taru-server`, `docs/api`
- Validation: focused catalog/image HTTP tests; OpenAPI route/schema tests;
  `cargo nextest run -p taru-server image --no-fail-fast`;
  `cargo nextest run -p taru-api image --no-fail-fast`; `git diff --check`
- Status: READY
- Review: replace public image DTOs and add first-party byte serving without
  leaking storage/source/cache/path locators
- Evidence: `EVIDENCE_AND_GATES.md`

## Blockers

- None known.

## Next Recommended Action

- Run MAPS-040 by replacing Public Client image DTOs with `PublicImageRefDto`,
  listing selected artwork from catalog item responses, adding `GET/HEAD
  /images/{image_id}`, and reading bytes through the internal managed artifact
  store.
- Remove or confine `ImageAssetDto`, `ImageRefDto.uri`, and
  `CanonicalMetadataDto.images` away from Public Client protocol/OpenAPI.
- Keep public image serving redacted: no `storage_uri`, source URL, `cache_uri`,
  local path, `managed-artwork://...`, or addon/provider token material.
- Keep thumbnails, durable retry/requeue, cancellation, and orphan artifact
  cleanup split from the first serving path.
