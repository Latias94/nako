# Managed Artwork Public Serving Selection Handoff

Status: Active
Last updated: 2026-05-19

## Current State

MAPS is open. The previous MAFA lane stores validated Managed Artwork bytes as
internal artifacts and keeps `managed-artwork://...` storage authority private.
This lane now owns the public boundary: Selected Artwork publication, redacted
Public Client image references, and first-party image byte serving. MAPS-020
froze the contract, MAPS-030 shipped explicit Admin publication, and MAPS-040
shipped Public Client image references plus first-party byte serving:

- `selected_artworks.id` is the public image ID authority.
- `POST /admin/v1/artwork/artifacts/{artifact_id}/publish` publishes a stored
  artifact as Selected Artwork.
- `PublicImageRefDto` replaces leak-prone Public Client image DTOs in item
  detail and item image listing responses.
- `ImageAsset` remains internal/provenance only and is not the public selected
  artwork authority.
- `GET /images/{image_id}` and `HEAD /images/{image_id}` serve selected
  artwork bytes through Taru-owned routes.
- Public Client protocol/OpenAPI no longer define `ImageAssetDto`,
  `ImageRefDto`, or `CanonicalMetadataDto.images`.

## Current Task

- Task ID: MAPS-050
- Owner: planner
- Files: `docs/workstreams/managed-artwork-public-serving-selection`, `docs/api`
- Validation: verify-rust-workstream records fresh final gate evidence
- Status: READY
- Review: close this lane or split thumbnails, durable retry/requeue, ingest
  cancellation, orphan artifact cleanup, and public gallery behavior into
  follow-ons
- Evidence: `EVIDENCE_AND_GATES.md`

## Blockers

- None known.

## Next Recommended Action

- Run MAPS-050 after final verification. Close the lane if no blocking review
  findings remain, or split thumbnails, durable retry/requeue, ingest
  cancellation, orphan artifact cleanup, and public gallery/candidate
  management into separate workstreams.
- Keep the public serving redaction invariant: no `storage_uri`, source URL,
  `cache_uri`, local path, `managed-artwork://...`, or addon/provider token
  material in Public Client/Admin DTOs.
