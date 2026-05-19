# Managed Artwork Public Serving Selection Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "ImageAssetDto|ImageRefDto|source_uri|cache_uri|storage_uri|selected|managed_artwork_artifacts|list_item_images|/items/\\{item_id\\}/images|/images" crates docs
git diff --check
```

This inventory proves the current public image DTOs, selected-artwork language,
managed artifact tables, and public catalog image routes are visible before the
redacted public-serving contract is changed.

## Gate Set

### Audit Gate

```powershell
rg -n "ImageAssetDto|ImageRefDto|source_uri|cache_uri|storage_uri|selected|managed_artwork_artifacts|list_item_images|/items/\\{item_id\\}/images|/images" crates docs
git diff --check
```

### Publication Gate

```powershell
cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests
cargo nextest run -p taru-db artwork --no-fail-fast
cargo nextest run -p taru-server artwork --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### Public Serving Gate

```powershell
cargo nextest run -p taru-server image --no-fail-fast
cargo nextest run -p taru-api image --no-fail-fast
cargo check -p taru-core -p taru-db -p taru-api -p taru-client-protocol -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

Adjust focused filters after MAPS-020 freezes concrete type and route names.

### Closeout Gate

```powershell
rg -n "source_uri|cache_uri|storage_uri|ImageAssetDto|ImageRefDto|selected" crates/taru-api crates/taru-client-protocol crates/taru-server/src/http docs/api
cargo check -p taru-core -p taru-db -p taru-api -p taru-client-protocol -p taru-server --tests
cargo nextest run -p taru-server image --no-fail-fast
cargo nextest run -p taru-api image --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Closeout must explain any remaining hits as internal/Admin-only or follow-on
work; unresolved Public Client leaks block completion.

## Evidence Anchors

- `docs/workstreams/managed-artwork-public-serving-selection/DESIGN.md`
- `docs/workstreams/managed-artwork-public-serving-selection/TODO.md`
- `docs/workstreams/managed-artwork-fetch-artifact-storage/HANDOFF.md`
- `crates/taru-core/src/media/artwork.rs`
- `crates/taru-core/src/media/catalog.rs`
- `crates/taru-core/src/repository/catalog.rs`
- `crates/taru-core/src/repository/metadata.rs`
- `crates/taru-db/migrations/0026_managed_artwork_ingest.sql`
- `crates/taru-db/src/artwork.rs`
- `crates/taru-db/src/catalog.rs`
- `crates/taru-api/src/public_client.rs`
- `crates/taru-api/src/openapi.rs`
- `crates/taru-client-protocol/src/catalog.rs`
- `crates/taru-server/src/app/artwork.rs`
- `crates/taru-server/src/app/catalog.rs`
- `crates/taru-server/src/http/catalog.rs`

## Fresh Evidence

2026-05-19, MAPS-010:

- Workstream opened from MAFA-050 closeout as the follow-on for public image
  references and Selected Artwork publication.
- Current audit findings:
  - `crates/taru-client-protocol/src/catalog.rs` exposes `ImageAssetDto` with
    `source_uri`, `cache_uri`, and `selected`.
  - `crates/taru-client-protocol/src/catalog.rs` exposes `ImageRefDto.uri`
    inside `CanonicalMetadataDto.images`.
  - `crates/taru-api/src/public_client.rs::image_asset_to_dto` copies
    `source_uri` and `cache_uri` into the Public Client DTO.
  - `crates/taru-api/src/openapi.rs` documents those fields in
    `ImageAssetDto`.
  - `crates/taru-server/src/app/catalog.rs::list_item_images` still returns
    public image responses from catalog `ImageAsset` rows.
  - `crates/taru-db/migrations/0026_managed_artwork_ingest.sql` already stores
    Managed Artwork Artifact metadata, but it has no Selected Artwork
    publication table.
- Scope decision:
  - use a separate Selected Artwork publication model instead of treating
    `image_assets.selected` as the new public source of truth;
  - serve image bytes through first-party Public Client routes;
  - keep `managed-artwork://...`, local artifact paths, raw source URLs, and
    cache URIs out of Public Client and redacted Admin responses;
  - split thumbnails, durable retry/requeue, cancellation, and orphan artifact
    cleanup.
- MAPS-010 validation:
  - `Get-Content -Raw docs/workstreams/managed-artwork-public-serving-selection/WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    passed.
  - `rg -n "ImageAssetDto|ImageRefDto|source_uri|cache_uri|storage_uri|selected|managed_artwork_artifacts|list_item_images|/items/\\{item_id\\}/images|/images" crates docs`
    produced 618 inventory lines for the public image, selected-artwork, and
    managed artifact seams.
  - `git diff --check` passed with only a Git CRLF normalization warning for
    `docs/workstreams/README.md`.

2026-05-19, MAPS-020 public contract and selection model freeze:

- Audited current code and docs before implementation:
  - `crates/taru-core/src/id.rs` has `ManagedArtworkArtifactId` and
    `ImageAssetId`, but no `SelectedArtworkId` yet.
  - `crates/taru-db/src/migrations.rs` currently stops at migration 0026, so
    the first Selected Artwork migration will be
    `0027_selected_artwork_publication.sql`.
  - `crates/taru-client-protocol/src/lib.rs` owns the Public Client route
    inventory; `/items/{item_id}/images` is currently JSON and no
    `/images/{image_id}` binary route exists.
  - `crates/taru-db/src/catalog.rs::list_item_images` still reads
    `image_assets` rows ordered by `selected`, which is not the future public
    selection authority.
  - `crates/taru-server/src/app/artwork.rs` has write/delete helpers for local
    Managed Artwork Artifact storage, but no read/stream helper yet.
  - `crates/taru-server/src/http/admin.rs` currently exposes candidate accept
    and ingest process-next routes, but no artifact publish route yet.
- Frozen contract:
  - public image ID authority is `selected_artworks.id`, represented as
    `SelectedArtworkId`;
  - Admin publication route is
    `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`;
  - Public Client byte routes are `GET /images/{image_id}` and
    `HEAD /images/{image_id}`;
  - Public Client image DTO is `PublicImageRefDto` with only selected-artwork
    ID, owner, kind, first-party relative URL, dimensions, language, media
    type, and safe ETag;
  - `ImageAssetDto`, `ImageRefDto.uri`, and `CanonicalMetadataDto.images` must
    leave the Public Client protocol path during MAPS-040;
  - legacy `ImageAsset` remains internal/provenance only until a later cleanup
    decides whether to migrate or delete it.
- Redaction decision:
  - Public and Admin responses must not expose `storage_uri`, local paths,
    `managed-artwork://...`, `source_uri`, `cache_uri`, raw provider URLs,
    Source Locators, addon token material, or provider query strings.
- Split decision:
  - thumbnail generation, durable retry/requeue, ingest cancellation, and
    orphan artifact cleanup remain separate follow-ons.
- `docs/api/HTTP_API.md` now documents the planned MAPS contract as planned,
  not as a current route.
- MAPS-020 validation:
  - `Get-Content -Raw docs/workstreams/managed-artwork-public-serving-selection/WORKSTREAM.json | ConvertFrom-Json | Out-Null`
    passed.
  - `rg -n "ImageAssetDto|ImageRefDto|source_uri|cache_uri|storage_uri|selected|managed_artwork_artifacts|list_item_images|/items/\\{item_id\\}/images|/images" crates docs`
    produced 689 inventory lines for the public image, selected-artwork, and
    managed artifact seams.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited documentation files.
