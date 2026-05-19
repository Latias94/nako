# Managed Artwork Thumbnail Variants Evidence And Gates

Status: Completed
Last updated: 2026-05-19

## Smallest Current Repro

```powershell
rg -n "/images/\\{image_id\\}|read_selected_image|PublicImageRefDto|ETAG|content_hash|variant|thumbnail|storage_uri|source_uri|cache_uri" crates docs
git diff --check
```

This inventory anchors the current public image route, DTO shape, image header
behavior, variant terms, and redaction boundaries.

## Gate Set

### Variant Serving Gate

```powershell
cargo nextest run -p taru-api managed_artwork_variant --no-fail-fast
cargo nextest run -p taru-server managed_artwork_variant --no-fail-fast
cargo nextest run -p taru-server public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks --no-fail-fast
cargo check -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

### Closeout Gate

```powershell
rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|local_path|artifact_root|variant|thumbnail" crates/taru-api crates/taru-server/src/http docs/api
cargo nextest run -p taru-api managed_artwork_variant --no-fail-fast
cargo nextest run -p taru-server managed_artwork_variant --no-fail-fast
cargo check -p taru-api -p taru-server --tests
cargo fmt --all -- --check
git diff --check
```

Remaining hits must be explained as internal storage logic, explicit redaction
assertions, route/query documentation, or tests proving forbidden values are
absent.

## Evidence Anchors

- `docs/workstreams/managed-artwork-public-serving-selection/HANDOFF.md`
- `crates/taru-api/src/public_client.rs`
- `crates/taru-api/src/openapi.rs`
- `crates/taru-server/src/app/artwork.rs`
- `crates/taru-server/src/http/catalog.rs`
- `crates/taru-server/src/http/tests/addons.rs`
- `docs/api/HTTP_API.md`

## Fresh Evidence

2026-05-19, MATV-010:

- Opened this lane from the Managed Artwork follow-on list.
- Scope decision:
  - first slice is explicit bounded query-parameter variants on selected image
    serving;
  - original image route stays compatible;
  - variants are on-demand only;
  - public validators must not expose artifact content hashes;
  - gallery, retry/cancel, missing repair, and persisted cache remain separate.

2026-05-19, MATV-020:

- Implemented bounded public image variants:
  - `GET/HEAD /images/{image_id}` now accept optional `width` and `height`
    query parameters;
  - no query parameters still serves original Selected Artwork bytes;
  - variants are derived on demand, preserve aspect ratio, avoid upscaling, and
    do not create variant DB rows or files;
  - on-demand variants are encoded as `image/png` bytes;
  - query dimensions are parsed as positive integers and capped by configured
    artwork image limits;
  - HTTP image ETags are opaque presentation validators instead of artifact
    content hashes;
  - `PublicImageRefDto.etag` is not populated from artifact content hash;
  - OpenAPI, generated TypeScript SDK, Rust client request builders, and
    `docs/api/HTTP_API.md` describe the variant query contract.
- Fresh validation:
  - `cargo nextest run -p taru-api managed_artwork_variant --no-fail-fast`:
    passed; 3 tests passed.
  - `cargo nextest run -p taru-server managed_artwork_variant --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -p taru-server public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -p taru-client streaming_request_builders_use_stable_paths_methods_headers_and_queries --no-fail-fast`:
    passed; 1 test passed.
  - `cargo nextest run -p taru-api sdk --no-fail-fast`: passed; 5 tests
    passed.
  - `cargo check -p taru-api -p taru-server -p taru-client --tests`: passed.
  - `npm run check --prefix sdk/typescript`: passed.
  - `cargo fmt --all -- --check`: passed.
  - `git diff --check`: passed.

2026-05-19, MATV-030:

- Closeout inventory:
  - `rg -n "storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|local_path|artifact_root|variant|thumbnail" crates/taru-api crates/taru-server/src/http docs/api`
    completed. Remaining hits are internal/Admin redaction fields, explicit
    forbidden-value assertions, route/query documentation, and tests proving
    forbidden values are absent.
- Lane split confirmation:
  - persisted variant cache and eviction remain a follow-on;
  - gallery/candidate management remains a follow-on;
  - durable retry/requeue/cancellation remains a follow-on;
  - missing-artifact repair or re-ingest remains a follow-on.
