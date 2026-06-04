# Evidence: selected artwork cache-control headers first slice

## Implementation Summary

Selected artwork image byte responses now include
`Cache-Control: private, max-age=86400` through a route-local helper in the
public catalog HTTP module. The helper is applied by `selected_image_response`,
so original GET, original HEAD, variant GET, and variant HEAD paths all share
the same private cache baseline while preserving existing content headers,
safe ETags, auth, library access checks, and variant behavior.

## Files Changed

* `crates/nako-server/src/http/catalog.rs`
* `crates/nako-server/src/http/tests/addons.rs`
* `.trellis/spec/nako-server/backend/http-api-patterns.md`
* `docs/architecture/CONTROL_PLANE.md`
* `docs/architecture/LIBRARY_PIPELINE.md`
* `.trellis/tasks/06-04-06-04-selected-artwork-cache-control-headers-first-slice/prd.md`
* `.trellis/tasks/06-04-06-04-selected-artwork-cache-control-headers-first-slice/research/selected-artwork-cache-control.md`

## Verification

* PASS: `cargo fmt --all -- --check`
* PASS: `cargo check -p nako-server --tests`
* PASS: `cargo nextest run -p nako-server public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks --no-fail-fast`
* PASS: `cargo nextest run -p nako-server managed_artwork_variant_routes_resize_selected_artwork_without_locator_or_hash_leaks --no-fail-fast`
* PASS: `git diff --check`
* PASS: `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-selected-artwork-cache-control-headers-first-slice`

## Notes

* An initial `cargo nextest` invocation used the helper name
  `assert_selected_artwork_variant_serving_without_locator_or_hash_leaks` and
  returned `0 tests run`; the PRD verification plan was corrected to the real
  route test names.
* `trellis-implement` and `trellis-check` sub-agents were attempted but timed
  out without writing code changes; both were closed before main-thread
  implementation/check continued.
* Out of scope remains unchanged: no conditional GET / 304, no immutable or
  shared-cache policy, no DTO/generated-contract/schema changes, and no HLS,
  Direct Play, Remux, or Admin cache behavior changes.
