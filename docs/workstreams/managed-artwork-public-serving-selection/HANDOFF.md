# Managed Artwork Public Serving Selection Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

MAPS is closed. The previous MAFA lane stored validated Managed Artwork bytes
as internal artifacts and kept `managed-artwork://...` storage authority
private. This lane completed the public boundary: Selected Artwork
publication, redacted Public Client image references, and first-party image byte
serving.

- `selected_artworks.id` is the public image ID authority.
- `POST /admin/v1/artwork/artifacts/{artifact_id}/publish` publishes a stored
  artifact as Selected Artwork.
- `PublicImageRefDto` replaces leak-prone Public Client image DTOs in item
  detail and item image listing responses.
- `ImageAsset` remains internal/provenance only and is not the public selected
  artwork authority.
- `GET /images/{image_id}` and `HEAD /images/{image_id}` serve selected
  artwork bytes through Nako-owned routes.
- Public Client protocol/OpenAPI no longer define `ImageAssetDto`,
  `ImageRefDto`, or `CanonicalMetadataDto.images`.

## Closeout

- Task ID: MAPS-050
- Owner: planner
- Files: `docs/workstreams/managed-artwork-public-serving-selection`, `docs/api`
- Validation: fresh closeout gates recorded in `EVIDENCE_AND_GATES.md`
- Status: DONE
- Review: no blocking workstream compliance findings remain
- Evidence: `EVIDENCE_AND_GATES.md`

## Blockers

- None.

## Follow-On Work

- `managed-artwork-thumbnail-variants`: thumbnail/resize generation,
  responsive variants, cache validators, and range/variant serving policy.
- `managed-artwork-ingest-runtime-controls`: durable retry/requeue,
  cancellation, and Admin/runtime controls for managed artwork ingest jobs.
- `managed-artwork-artifact-lifecycle-cleanup`: orphan artifact detection,
  selected-artwork retention protection, artifact garbage collection, and
  operator diagnostics.
- `managed-artwork-gallery-candidate-management`: public/Admin browsing for
  candidates and artwork galleries after the Selected Artwork boundary is
  stable.

Keep the redaction invariant in all follow-ons: no `storage_uri`, source URL,
`cache_uri`, local path, `managed-artwork://...`, or addon/provider token
material in Public Client/Admin DTOs.
