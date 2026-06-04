# Lane C: Selected Artwork Delivery Cache First Slice

Date: 2026-06-04
Agent: trellis-implement

## Scope

Implemented a metadata-only selected artwork ETag preflight for authenticated
Public Client selected artwork image routes.

Touched code is limited to:

- `crates/nako-server/src/app/artwork.rs`
- `crates/nako-server/src/app/artwork/variant.rs`
- `crates/nako-server/src/http/catalog.rs`
- selected artwork HTTP tests in `crates/nako-server/src/http/tests/addons.rs`

## Behavior

- `GET /images/{image_id}` and `HEAD /images/{image_id}` still run auth and
  Library Access checks before any 304 response.
- If `If-None-Match` exactly matches a metadata-derived selected artwork ETag,
  the route returns `304 Not Modified` with the existing private artwork cache
  header and no body.
- Original image ETags can be derived from selected/artifact metadata.
- Bounded variant ETags can be derived only when artifact width and height
  metadata are present and valid.
- If preflight cannot prove the current ETag, the route falls back to the
  existing byte read and variant derivation path.

## Deferred

- No schema changes.
- No Admin/Public DTO changes.
- No generated contract changes.
- No durable binary derivative store.
- No placeholder or blurhash contract.

## Validation

- `rustfmt --edition 2024 crates/nako-server/src/app/artwork.rs crates/nako-server/src/app/artwork/variant.rs crates/nako-server/src/http/catalog.rs crates/nako-server/src/http/tests/addons.rs`
  - Passed.
- `cargo nextest run -p nako-server catalog --no-fail-fast`
  - Passed: 32 passed, 562 skipped.
  - Noted warning from parallel playback work:
    `missing_playback_transcode_error` is unused in
    `crates/nako-server/src/app/playback/runtime_session.rs`.
- `cargo nextest run -p nako-server artwork --no-fail-fast`
  - Passed: 19 passed, 575 skipped.
- `cargo check -p nako-server --tests`
  - Passed.
- `rustfmt --edition 2024 --check crates/nako-server/src/app/artwork.rs crates/nako-server/src/app/artwork/variant.rs crates/nako-server/src/http/catalog.rs crates/nako-server/src/http/tests/addons.rs`
  - Passed.
- `git diff --check -- crates/nako-server/src/app/artwork.rs crates/nako-server/src/app/artwork/variant.rs crates/nako-server/src/http/catalog.rs crates/nako-server/src/http/tests/addons.rs .trellis/tasks/06-04-10-hour-media-server-architecture-campaign/implementation/lane-c-artwork-cache.md`
  - Passed.
