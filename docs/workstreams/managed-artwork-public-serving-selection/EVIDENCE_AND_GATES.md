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
