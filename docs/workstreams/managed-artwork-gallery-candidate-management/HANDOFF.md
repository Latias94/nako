# Managed Artwork Gallery Candidate Management Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This lane is open and `MAGC-020` is complete.

The lane owns Admin management for item-scoped artwork choices after the
candidate ingest, artifact storage, selected artwork publication, public image
serving, lifecycle cleanup, remediation policy, and thumbnail variant lanes.

## Next Step

Implement `MAGC-030`:

- decide whether selection management should keep using
  `POST /admin/v1/artwork/artifacts/{artifact_id}/publish` or add an
  item/kind-scoped command such as `POST /admin/v1/items/{item_id}/artwork/{kind}/select`;
- preserve idempotent replacement semantics;
- ensure the chosen command cannot select an artifact for the wrong item/kind;
- keep artifact deletion, file cleanup, unpublish, retry/cancel, and repair out
  of the action unless explicitly split;
- prove Public Client item images reflect the selected artifact through
  first-party `/images/{image_id}` references.

## Blockers

None known.

## Completed In MAGC-020

- Added a core/db gallery snapshot that does not carry raw candidate
  `source_uri`, artifact `storage_uri`, or content hash values into the Admin
  response path.
- Added explicit Admin DTOs:
  - `AdminManagedArtworkGalleryResponse`;
  - `AdminManagedArtworkGalleryCandidate`;
  - `AdminManagedArtworkGalleryArtifact`;
  - `AdminManagedArtworkGallerySelected`.
- Added `GET /admin/v1/items/{item_id}/artwork?limit=50&offset=0`.
- Updated `ProcessManagedArtworkIngestResponse` artifact summaries to expose
  `has_content_hash` rather than content hash values.
- Updated `docs/api/HTTP_API.md`.

## Follow-Ons Outside This Lane

- Public Client candidate/gallery browsing.
- Persisted thumbnail/variant cache and eviction.
- `managed-artwork-ingest-runtime-controls`.
- Missing-artifact repair/re-ingest.
- Provider search, scraping, or automatic artwork ranking.

Keep the redaction invariant in all follow-ons: no `storage_uri`, source URL,
`source_uri`, `cache_uri`, local path, `managed-artwork://...`, artifact content
hash, file contents, or addon/provider token material in Public Client/Admin
DTOs.
