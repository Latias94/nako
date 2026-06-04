# Evidence: selected artwork conditional GET first slice

## Implementation Summary

Selected artwork public image GET/HEAD handlers now accept request headers and
pass `If-None-Match` into the shared selected artwork response builder. When the
request header exactly matches the current quoted safe ETag, the response is
`304 Not Modified` with the selected artwork private cache policy and current
ETag. Missing, malformed, or non-matching validators preserve existing 200
GET/HEAD behavior.

## Files Changed

* `crates/nako-server/src/http/catalog.rs`
* `crates/nako-server/src/http/tests/addons.rs`
* `.trellis/spec/nako-server/backend/http-api-patterns.md`
* `docs/architecture/CONTROL_PLANE.md`
* `docs/architecture/LIBRARY_PIPELINE.md`
* `.trellis/tasks/06-04-06-04-selected-artwork-conditional-get-first-slice/prd.md`
* `.trellis/tasks/06-04-06-04-selected-artwork-conditional-get-first-slice/research/selected-artwork-conditional-get.md`

## Verification

* PASS: `cargo fmt --all -- --check`
* PASS: `cargo check -p nako-server --tests`
* PASS: `cargo nextest run -p nako-server public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks --no-fail-fast`
* PASS: `cargo nextest run -p nako-server managed_artwork_variant_routes_resize_selected_artwork_without_locator_or_hash_leaks --no-fail-fast`
* PASS: `git diff --check`
* PASS: `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-selected-artwork-conditional-get-first-slice`

## Notes

* The first slice intentionally compares after deriving the current image bytes.
  Metadata-only ETag preflight can avoid artifact reads/resizing later, but that
  would require a separate app-service seam.
* `trellis-implement` was attempted and timed out without writing code changes;
  it was closed before main-thread implementation continued.
* Out of scope remains unchanged: no weak/wildcard validators, no
  `Last-Modified`, no immutable/shared-cache policy, no DTO/generated-contract
  or schema changes, and no HLS, Direct Play, Remux, or Admin behavior changes.
