# Selected Artwork Unpublish Delete Policy Design

Status: Completed
Last updated: 2026-05-19

## Problem

Selected Artwork now owns public image identity for item artwork. Admin users
can publish stored artifacts and replace an item/kind selection, but there is no
explicit operation for removing a Selected Artwork slot.

Without a first-class unpublish boundary, future UI or cleanup work can make two
dangerous mistakes:

- treating "remove this public image" as "delete the artifact bytes";
- treating "artifact cleanup candidate" as an immediate side effect of an Admin
  selection change.

Those operations have different authorities. `selected_artworks` controls public
visibility and retention protection, `managed_artwork_artifacts` records stored
validated artifacts, and the artifact store owns physical bytes. This lane keeps
those authorities separate.

## Target State

- Admin API exposes one explicit item/kind unpublish command.
- Unpublish removes the Selected Artwork row for the requested slot.
- Unpublish does not delete the linked Managed Artwork Artifact row.
- Unpublish does not delete local artifact files.
- Public item image lists stop returning the unpublished slot.
- The previously issued public image ID stops resolving and returns `404`.
- The previously selected artifact is considered by artifact lifecycle cleanup
  only after the repository observes no remaining Selected Artwork references.
- Admin responses include safe IDs and state-change facts, but never storage
  locators, source URLs, local paths, cache handles, or content hashes.

## Route Direction

The preferred route is:

```text
DELETE /admin/v1/items/{item_id}/artwork/{kind}/selection
```

The route is item/kind-scoped because the operator intent is "remove this
published artwork slot from this item." It should not accept artifact IDs, raw
URLs, storage handles, or paths in the request body.

Expected behavior:

- existing item and selected slot: `200` with `changed = true`;
- existing item and no selected slot: `200` with `changed = false`;
- missing item: `404`;
- invalid artwork kind: `400`;
- internal artifact/file deletion: never performed by this route.

The route is idempotent at the selection-slot level so Admin UI retries do not
turn an already-unpublished slot into an error.

## Public Image Behavior

The publication lane made `selected_artworks.id` the public image ID authority.
After unpublish, that selected row no longer exists. Therefore:

```text
GET  /images/{old_selected_id}  -> 404
HEAD /images/{old_selected_id}  -> 404
```

This is intentional. Taru keeps stable image IDs while a slot remains selected
and may preserve the ID across replacement, but unpublish removes the public
publication. Artifact IDs are not public image IDs and must not be used as a
fallback.

## Retention And Cleanup Policy

Unpublish changes only Selected Artwork state. It must not call artifact cleanup
or file deletion directly.

After unpublish:

- the Managed Artwork Artifact row remains available for Admin gallery display;
- the stored bytes remain in the artifact store;
- lifecycle diagnostics may report the artifact as a cleanup candidate if no
  Selected Artwork rows reference it;
- cleanup can later mark/delete the artifact through the existing protected
  cleanup command;
- `ON DELETE RESTRICT` and repository guards continue to protect still-selected
  artifacts.

This preserves a clear recovery path: an operator can unpublish an image, inspect
the item gallery, and reselect the same artifact before any separate cleanup
decision is made.

## Response Shape

The shipped response is explicit and redacted:

```text
UnpublishSelectedArtworkResponse {
  item_id: string,
  kind: string,
  changed: bool,
  unpublished: UnpublishedSelectedArtworkSummary | null
}

UnpublishedSelectedArtworkSummary {
  selected_artwork: SelectedArtworkSummary,
  previous_image: PublicImageRefDto
}
```

`previous_image.url` is safe because it is a first-party route derived from the
previous Selected Artwork ID. The response does not claim the URL is still
fetchable; it is an audit/UI fact about what publication was removed.

Do not include:

- `storage_uri`;
- `managed-artwork://...`;
- local artifact root paths;
- raw candidate source URLs;
- `source_uri`;
- `cache_uri`;
- provider query strings;
- addon tokens or provider credentials;
- file contents;
- content-hash values.

## Architecture Direction

- `taru-core` owns the domain result type and repository method for unpublish.
- `taru-db` implements an item/kind-scoped delete guarded by item existence and
  returning the previous Selected Artwork facts when a row existed.
- `taru-api` owns explicit Admin DTOs for the command response.
- `taru-server::app::artwork` orchestrates validation, repository calls, and
  redacted DTO construction.
- `taru-server::http::admin` owns route parsing and error mapping only.
- Public Client DTOs stay unchanged; they should naturally stop returning the
  selection because they read from `selected_artworks`.

## Assumptions

| Assumption | Confidence | Evidence | Mitigation |
| --- | --- | --- | --- |
| Deleting the Selected Artwork row is the correct unpublish model. | High | Public image ID authority is `selected_artworks.id`; cleanup already uses Selected Artwork references for retention. | Document `404` for old public IDs and add route regression tests. |
| Idempotent unpublish is better for Admin UI retries. | Medium | Selection replacement is already idempotent where possible. | Keep missing item as `404`; only no-current-selection returns `changed = false`. |
| Artifact bytes should survive unpublish. | High | Existing lifecycle cleanup is a separate explicit Admin operation. | Add DB/server tests proving artifact lookup/gallery still sees the artifact after unpublish. |

## Splits

- Artifact deletion and physical cleanup stay in the lifecycle cleanup and
  remediation lanes.
- Public candidate/gallery browsing remains outside this lane.
- Variant cache persistence and eviction remain outside this lane.
- Durable ingest retry/requeue/cancellation remains outside this lane.
- Missing-artifact repair or re-ingest remains outside this lane.

## Closeout Condition

This lane can close when Admin users can unpublish item/kind Selected Artwork
without deleting artifacts, Public Client item images and image byte routes
reflect the unpublished state, artifact cleanup eligibility remains explicit,
docs describe the lifecycle boundary, and fresh validation evidence proves
redaction and retention behavior.

Status: Closed. The shipped command is
`DELETE /admin/v1/items/{item_id}/artwork/{kind}/selection`. It removes the
Selected Artwork row, keeps the Managed Artwork Artifact record and bytes out of
the unpublish side effect, makes old image IDs resolve to `404`, and leaves
future deletion to explicit lifecycle cleanup.
