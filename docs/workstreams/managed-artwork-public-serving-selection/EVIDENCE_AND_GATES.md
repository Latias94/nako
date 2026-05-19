# Managed Artwork Public Serving Selection Evidence And Gates

Status: Completed
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

2026-05-19, MAPS-030 Selected Artwork publication:

- Added `SelectedArtworkId` to `taru-core`.
- Added `SelectedArtworkRecord` and `SelectedArtworkPublicationRecord`.
- Added `ManagedArtworkRepository` methods:
  `publish_selected_artwork`, `get_selected_artwork`, and
  `list_selected_artwork_for_item`.
- Added migration `0027_selected_artwork_publication.sql`:
  - `selected_artworks.id` is the stable public image identity;
  - unique `(item_id, kind, kind_key)` keeps one selected artifact per item/kind
    slot;
  - `artifact_id` references `managed_artwork_artifacts(id)` with
    `ON DELETE RESTRICT`;
  - artifact and item indexes support serving follow-ons.
- Added SQLite repository behavior:
  - publishing requires a stored Managed Artwork Artifact linked to a stored
    ingest;
  - publishing the same artifact again returns the same Selected Artwork ID and
    `changed = false`;
  - publishing preserves artifact storage authority internally and returns no
    `storage_uri`.
- Added `PublicImageRefDto` as the redacted public image reference shape for
  publication responses.
- Added `PublishSelectedArtworkResponse` and Admin route
  `POST /admin/v1/artwork/artifacts/{artifact_id}/publish`.
- Added HTTP/API docs for the current Admin publish route while keeping
  `GET/HEAD /images/{image_id}` documented as planned MAPS-040 work.
- Redaction evidence:
  - Admin publication response omits `storage_uri`, `managed-artwork://...`,
    source URL, `source_uri`, `cache_uri`, local artifact path, raw token, and
    provider query string material.
- Validation:
  - `cargo nextest run -p taru-api selected_artwork_publication_response_redacts_storage_uri --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-db sqlite_store_publishes_stored_managed_artifact_as_selected_artwork_idempotently --no-fail-fast`
    passed.
  - `cargo nextest run -p taru-server admin_publish_managed_artwork_artifact_creates_selected_artwork_without_locator_leaks --no-fail-fast`
    passed.
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`
    passed.
  - `cargo nextest run -p taru-db artwork --no-fail-fast` passed: 4 tests.
  - `cargo nextest run -p taru-server artwork --no-fail-fast` passed: 7 tests.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.

2026-05-19, MAPS-040 Public image references and byte serving:

- Replaced Public Client catalog image responses:
  - `ItemDetailResponse.images` now serializes `Vec<PublicImageRefDto>`;
  - `ImagesResponse.images` now serializes `Vec<PublicImageRefDto>`;
  - `CanonicalMetadataDto` no longer serializes provider image URI records.
- Removed public `ImageAssetDto` and `ImageRefDto` from
  `taru-client-protocol` and the Public OpenAPI schemas. Legacy `ImageAsset`
  remains internal/provenance only.
- Added Public Client route inventory and OpenAPI operations for:
  - `GET /images/{image_id}`;
  - `HEAD /images/{image_id}`.
- Added Rust SDK request builders and regenerated the committed TypeScript SDK
  so both SDKs expose the selected artwork byte route without old raw image DTOs.
- Updated `CatalogAppService` to list selected artwork records and build
  first-party `PublicImageRefDto` values from selected artifact metadata.
- Added server-side selected image serving:
  - resolves `selected_artworks.id` to a Managed Artwork Artifact;
  - validates the internal `managed-artwork://artifact/{artifact_id}` authority
    inside the server boundary;
  - reads bytes only below the configured artifact root;
  - returns `Content-Type`, `Content-Length`, and quoted ETag when a safe
    content hash exists.
- Redaction evidence:
  - public catalog/image HTTP tests reject source URL, provider query string,
    addon raw token, `source_uri`, `cache_uri`, `storage_uri`,
    `managed-artwork://...`, and local artifact root leakage;
  - OpenAPI image contract tests prove `ImageAssetDto`, `ImageRefDto`, and
    `CanonicalMetadataDto.images` are absent from the Public Client contract.
- Validation:
  - `cargo nextest run -p taru-api image --no-fail-fast` passed.
  - `cargo nextest run -p taru-server image --no-fail-fast` passed.
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-client-protocol -p taru-client -p taru-server --tests` passed.
  - `cargo nextest run -p taru-client streaming_request_builders_use_stable_paths_methods_headers_and_queries sdk_inventory_uses_shared_protocol_routes_and_exposure --no-fail-fast` passed.
  - `cargo nextest run -p taru-client-protocol public_route_inventory_is_protocol_owned_and_complete public_browse_dtos_use_wire_ids_and_client_enums --no-fail-fast` passed.
  - `cargo nextest run -p taru-api sdk --no-fail-fast` passed.
  - `npm run check --prefix sdk/typescript` passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with only Git CRLF normalization warnings for
    edited files.
  - `rg -n "ImageAssetDto|ImageRefDto|source_uri|cache_uri|storage_uri|managed-artwork://|CanonicalMetadataDto.*images" crates/taru-client-protocol/src crates/taru-api/src/openapi.rs crates/taru-api/src/public_client.rs crates/taru-api/src/sdk.rs sdk/typescript/src/index.ts crates/taru-server/src/http/catalog.rs crates/taru-server/src/app/catalog.rs docs/api/HTTP_API.md`
    showed no old Public Client image DTOs in protocol/OpenAPI/SDK output.
    Remaining sensitive-term hits are redaction assertions, internal server
    storage resolution, or HTTP/API text that states the values are forbidden
    in public responses.

2026-05-19, MAPS-050 closeout verification:

- Closeout audit:
  - target state is met: stored Managed Artwork Artifacts can be published as
    Selected Artwork, Public Client image references are first-party and
    redacted, and selected image bytes are served through Taru-owned routes;
  - `ImageAsset` remains internal/provenance only;
  - thumbnails, durable retry/requeue, cancellation, orphan cleanup, and
    gallery/candidate management are split as follow-ons.
- Fresh gate evidence:
  - `rg -n "source_uri|cache_uri|storage_uri|ImageAssetDto|ImageRefDto|selected" crates/taru-api crates/taru-client-protocol crates/taru-server/src/http docs/api`
    passed as an inventory. Remaining hits are Admin/internal tests, redaction
    assertions, HTTP/API prohibitions, non-artwork playback wording, or
    `PublicImageRefDto` selected-artwork references; no old Public Client raw
    image DTO remains in protocol/OpenAPI.
  - `cargo check -p taru-core -p taru-db -p taru-api -p taru-client-protocol -p taru-client -p taru-server --tests`
    passed.
  - `cargo nextest run -p taru-server image --no-fail-fast` passed.
  - `cargo nextest run -p taru-api image --no-fail-fast` passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed.
  - `npm run check --prefix sdk/typescript` passed.
