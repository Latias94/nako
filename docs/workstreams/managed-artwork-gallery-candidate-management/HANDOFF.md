# Managed Artwork Gallery Candidate Management Handoff

Status: Active
Last updated: 2026-05-19

## Current State

This lane is open and `MAGC-030` is complete.

The lane owns Admin management for item-scoped artwork choices after the
candidate ingest, artifact storage, selected artwork publication, public image
serving, lifecycle cleanup, remediation policy, and thumbnail variant lanes.

## Next Step

Implement `MAGC-040`:

- run closeout verification gates;
- decide whether remaining work belongs in this lane or split follow-ons;
- update `EVIDENCE_AND_GATES.md`, `MILESTONES.md`, `WORKSTREAM.json`, and this
  handoff;
- keep unpublish/delete behavior out of closeout unless a separate retention
  policy is designed and tested.

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

## Completed In MAGC-030

- Added guarded repository method
  `publish_selected_artwork_for_item_kind(item_id, kind, artifact_id)`.
- Added `POST /admin/v1/items/{item_id}/artwork/{kind}/select`.
- Preserved idempotent replacement semantics and the existing Selected Artwork
  public ID for the slot.
- Rejected artifact selection when the artifact belongs to another item or
  image kind.
- Verified Public Client item image listing reflects the newly selected
  artifact.

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
