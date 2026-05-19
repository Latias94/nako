# Managed Artwork Gallery Candidate Management Evidence And Gates

Status: Completed
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "ArtworkCandidate|artwork_candidate|addon_artwork_candidates|SelectedArtwork|selected_artwork|PublicImageRefDto|publish_selected_artwork|/items/\\{item_id\\}/images|artwork/artifacts/.*/publish|storage_uri|source_uri|cache_uri|content_hash|gallery" crates docs
git diff --check
```

This inventory anchors the current candidate, artifact, selected artwork,
public image reference, and redaction boundaries before adding a management
read model.

## Gate Set

### Gallery Read Model Gate

```powershell
cargo nextest run -p taru-api managed_artwork_gallery --no-fail-fast
cargo nextest run -p taru-db managed_artwork_gallery --no-fail-fast
cargo nextest run -p taru-server managed_artwork_gallery --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

### Selection Management Gate

```powershell
cargo nextest run -p taru-db managed_artwork_gallery --no-fail-fast
cargo nextest run -p taru-server managed_artwork_gallery --no-fail-fast
cargo nextest run -p taru-server public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

### Closeout Gate

```powershell
rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|local_path|artifact_root|gallery|candidate|selected_artwork" crates/taru-api crates/taru-server/src/http docs/api
cargo nextest run -p taru-api managed_artwork_gallery --no-fail-fast
cargo nextest run -p taru-db managed_artwork_gallery --no-fail-fast
cargo nextest run -p taru-server managed_artwork_gallery --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

Remaining hits must be explained as internal repository fields, explicit
redaction assertions, route documentation, or tests proving forbidden values are
absent.

## Evidence Anchors

- `docs/workstreams/managed-artwork-public-serving-selection/HANDOFF.md`
- `docs/workstreams/managed-artwork-thumbnail-variants/HANDOFF.md`
- `docs/workstreams/managed-artwork-remediation-policy/HANDOFF.md`
- `crates/taru-core/src/media/artwork.rs`
- `crates/taru-core/src/repository/metadata.rs`
- `crates/taru-db/src/artwork.rs`
- `crates/taru-api/src/admin.rs`
- `crates/taru-api/src/public_client.rs`
- `crates/taru-server/src/app/artwork.rs`
- `crates/taru-server/src/app/catalog.rs`
- `crates/taru-server/src/http/admin.rs`
- `crates/taru-server/src/http/catalog.rs`
- `docs/api/HTTP_API.md`

## Fresh Evidence

2026-05-19, MAGC-010:

- Opened this lane from the Managed Artwork follow-on list after
  `managed-artwork-thumbnail-variants` closed.
- Scope decision:
  - first slice is an Admin item-scoped artwork gallery read model;
  - selection management follows after the read model proves the terminology
    and redaction contract;
  - Public Client candidate/gallery browsing is deferred;
  - persisted variant cache, durable retry/cancel, missing repair, provider
    search/ranking, and deletion/unpublish policy stay out of this lane unless
    split explicitly.

2026-05-19, MAGC-020:

- Implemented `GET /admin/v1/items/{item_id}/artwork?limit=50&offset=0`.
- Added core/db gallery snapshot records that avoid carrying raw
  `source_uri`, `storage_uri`, or content hash values into the Admin response
  path.
- Added explicit Admin gallery DTOs and route docs.
- Tightened managed ingest artifact summaries to expose `has_content_hash`
  instead of content hash values.
- Fresh focused validation:
  - `cargo nextest run -p taru-api managed_artwork_gallery --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-db managed_artwork_gallery --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-server managed_artwork_gallery --no-fail-fast`
    passed.
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed.

2026-05-19, MAGC-030:

- Implemented guarded selection command:
  `POST /admin/v1/items/{item_id}/artwork/{kind}/select`.
- Added repository-level item/kind guard via
  `publish_selected_artwork_for_item_kind`.
- Preserved idempotent replacement semantics and Public Client image listing
  behavior.
- Fresh focused validation so far:
  - `cargo nextest run -p taru-db publishes_selected_artwork_with_item_kind_guard --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-db managed_artwork_gallery --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-server managed_artwork_gallery --no-fail-fast`
    passed.
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed.

2026-05-19, MAGC-040 closeout:

- Closeout claim:
  - item-scoped Admin gallery read model is implemented and documented;
  - item/kind-scoped selection management is implemented and documented;
  - Public Client image responses remain Selected Artwork only;
  - unpublish/delete, Public gallery browsing, persisted variant cache,
    retry/cancel, and repair/re-ingest are split follow-ons.
- Fresh closeout validation:
  - `rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|local_path|artifact_root|gallery|candidate|selected_artwork" crates/taru-api crates/taru-server/src/http docs/api`
    completed. Remaining hits are route names, docs stating forbidden fields,
    safe boolean fields such as `has_content_hash`, internal/admin tests, and
    redaction assertions.
  - `cargo nextest run -p taru-api managed_artwork_gallery --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-db managed_artwork_gallery --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-server managed_artwork_gallery --no-fail-fast`
    passed.
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed.
