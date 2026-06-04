# Evidence

## Changes

- Extended selected artwork `If-None-Match` matching in
  `crates/nako-server/src/http/catalog.rs` to support exact, weak,
  comma-separated validator-list, and wildcard forms against the current safe
  selected artwork ETag.
- Expanded selected artwork HTTP route tests in
  `crates/nako-server/src/http/tests/addons.rs` to cover original image,
  resized variant, malformed weak miss, wildcard, and metadata-derived preflight
  behavior.
- Updated `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `docs/api/HTTP_API.md`, `docs/architecture/CONTROL_PLANE.md`, and
  `docs/architecture/LIBRARY_PIPELINE.md` so weak/wildcard validators are part
  of the shipped selected artwork cache contract instead of a follow-on.

## Validation

- `cargo fmt --all`
- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-server selected_artwork_without_locator --no-fail-fast`
  passed: 3 selected artwork route tests passed.
- `cargo check -p nako-server --tests`
- `git diff --check` passed with only Git LF/CRLF working-copy warnings.
- Focused grep over active cache-contract docs found no old weak/wildcard
  follow-on wording for selected artwork.

## Spec Update Judgment

This task changed a public HTTP cache contract, so
`.trellis/spec/nako-server/backend/http-api-patterns.md` was updated in place.
No new generic guide was needed because the reusable lesson is already captured
as a concrete server route contract.
