# Selected Artwork Unpublish Delete Policy Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

This lane is complete.

The lane owns the lifecycle boundary between:

- Selected Artwork unpublish;
- Managed Artwork Artifact record retention;
- physical artifact byte cleanup;
- Public Client image visibility.

## Completion

Shipped:

- `SelectedArtworkUnpublicationRecord` in `taru-core`;
- `ManagedArtworkRepository::unpublish_selected_artwork_for_item_kind`;
- SQLite item/kind-scoped Selected Artwork unpublish;
- redacted `UnpublishSelectedArtworkResponse`;
- `DELETE /admin/v1/items/{item_id}/artwork/{kind}/selection`;
- HTTP docs for unpublish, retention, cleanup, and old image ID `404`
  behavior.

Verified behavior:

- item/kind-scoped command;
- idempotent for existing item/kind slots with no current selection;
- invalid kind returns `400`;
- no artifact record deletion;
- no file deletion;
- Public item image lists omit unpublished slots;
- old `GET` and `HEAD /images/{old_selected_id}` return `404`;
- linked artifacts become cleanup-eligible only through lifecycle rules;
- responses remain redacted.

## Key Policy Decisions

- Unpublish deletes/removes the Selected Artwork publication row.
- Unpublish does not delete the linked Managed Artwork Artifact row.
- Unpublish does not delete local artifact bytes.
- `GET` and `HEAD /images/{old_selected_id}` return `404` after unpublish.
- The previous artifact becomes cleanup-eligible only through existing lifecycle
  rules when no Selected Artwork rows reference it.

## Files To Inspect Next

None required for this lane.

## Suggested Validation

```powershell
$env:CARGO_TARGET_DIR='G:\taru-cargo-target'
cargo nextest run -p taru-api selected_artwork_unpublish --no-fail-fast
cargo nextest run -p taru-db selected_artwork_unpublish --no-fail-fast
cargo nextest run -p taru-server selected_artwork_unpublish --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

## Follow-Ons Outside This Lane

- Artifact deletion and physical cleanup policy changes.
- Missing-artifact repair or re-ingest.
- Public Client candidate/gallery browsing.
- Persisted thumbnail or variant cache eviction.
- Durable ingest retry/requeue/cancellation controls.
