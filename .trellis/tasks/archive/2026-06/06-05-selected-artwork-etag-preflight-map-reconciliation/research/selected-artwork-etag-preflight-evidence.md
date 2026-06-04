# Selected Artwork ETag Preflight Evidence

## Question

Has metadata-only selected artwork ETag preflight shipped, and what remains?

## Local Evidence

- Commit `b52d6594 feat(server): harden media server runtime slices` includes:
  - `crates/nako-server/src/app/artwork.rs`;
  - `crates/nako-server/src/app/artwork/variant.rs`;
  - `crates/nako-server/src/http/catalog.rs`;
  - selected artwork HTTP tests.
- Campaign implementation note:
  `.trellis/tasks/archive/2026-06/06-04-10-hour-media-server-architecture-campaign/implementation/lane-c-artwork-cache.md`.
- Server HTTP spec:
  `.trellis/spec/nako-server/backend/http-api-patterns.md` now documents
  `selected_image_preflight_response(...) -> Option<Response>` as the
  metadata-derived exact-match 304 short-circuit after auth and library access.
- Current code has `ArtworkAppService::selected_image_preflight` and
  `selected_image_preflight_response` before the selected image byte read path.

## Remaining Follow-Ons

- weak/wildcard validator parsing;
- `Last-Modified`;
- immutable or shared-cache/CDN semantics;
- selected-artwork invalidation policy;
- derivative generation, WebP/size presets, placeholders, and Blurhash.
