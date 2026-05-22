# Selected Artwork Unpublish Delete Policy

Status: Completed
Last updated: 2026-05-19

## Purpose

Nako can now ingest Managed Artwork Artifacts, publish them as Selected Artwork,
serve selected images through first-party image routes, generate bounded
variants, clean up unselected artifacts, and let Admin users compare and replace
item artwork choices. The missing lifecycle boundary is explicit unpublish:
operators need a safe way to remove a Selected Artwork publication without
accidentally deleting artifact bytes, exposing internal locators, or making
artifact cleanup semantics ambiguous.

## Goals

- Define the difference between unpublishing Selected Artwork, deleting Managed
  Artwork Artifacts, and physically removing stored bytes.
- Add an explicit Admin command for unpublishing an item/kind Selected Artwork
  slot.
- Keep Public Client image lists selected-artwork-only after unpublish.
- Define the public image route behavior for an unpublished Selected Artwork
  public ID.
- Preserve artifact retention: unpublish must not delete artifact records or
  files directly.
- Keep Admin/Public responses redacted.

## Non-Goals

- Deleting Managed Artwork Artifact records or files as part of unpublish.
- Adding artifact repair, re-ingest, retry, requeue, or cancellation controls.
- Public Client candidate/gallery browsing.
- Persisted thumbnail or variant cache eviction.
- Provider search, scraping, ranking, or automatic artwork selection.
- Exposing `storage_uri`, `managed-artwork://...`, local paths, raw source
  URLs, `source_uri`, `cache_uri`, provider query strings, addon tokens, file
  contents, or artifact content hashes.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Shipped

This lane shipped the explicit Selected Artwork unpublish boundary:

- `DELETE /admin/v1/items/{item_id}/artwork/{kind}/selection` is the preferred
  Admin command.
- The command removes the Selected Artwork publication slot only.
- It is idempotent for an existing item/kind slot with no current selection.
- It never deletes a Managed Artwork Artifact record or file.
- The previously selected artifact becomes cleanup-eligible only through the
  existing artifact lifecycle rules when no Selected Artwork rows reference it.
- `GET` and `HEAD /images/{old_selected_id}` return `404` after the selected row
  is unpublished because the public image identity is no longer published.

## Closeout

Completed in `SAUD-020`, `SAUD-030`, and `SAUD-040` with focused API, DB, and
server tests plus HTTP documentation updates.
