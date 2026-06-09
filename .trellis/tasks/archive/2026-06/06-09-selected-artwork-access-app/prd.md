# refactor: move selected artwork access into app service

## Goal

Move authenticated Public Client selected artwork image access enforcement out
of the catalog HTTP handlers and into the managed artwork app-service boundary,
while preserving existing selected artwork GET/HEAD cache, ETag, variant, and
error behavior.

## What I Already Know

* `GET /images/{image_id}` and `HEAD /images/{image_id}` currently call
  `require_selected_artwork_access(... Browse)` in `http::catalog`.
* Selected artwork image responses already have a route-specific private cache
  and conditional ETag contract.
* The existing order is auth, library access, variant query validation,
  metadata-derived preflight, then byte read.
* Access checks must still happen before any `304 Not Modified` response.
* This task follows the recent app-service access-boundary pattern used by
  user playlist item access and user playback access.

## Requirements

* `get_image` and `head_image` must pass `AuthenticatedPrincipal` into the
  app/service layer for selected artwork access enforcement.
* `ManagedArtworkAppService` must enforce Browse access for selected artwork
  reads and preflight lookups before returning image bytes or ETag preflight
  metadata.
* HTTP selected artwork handlers must remain thin: parse query/header/path
  input and assemble HTTP responses only.
* Preserve existing selected artwork cache headers, ETag matching semantics,
  body/no-body behavior, content headers, variant validation, not-found, and
  forbidden behavior.
* Preserve the current ordering where unauthorized callers receive `403`
  before variant query validation or ETag preflight can reveal image state.
* Remove the route-local selected artwork access helper if it has no remaining
  callers.

## Acceptance Criteria

* [x] `crates/nako-server/src/http/catalog.rs` no longer imports or calls
  `require_selected_artwork_access`.
* [x] Selected artwork app-service methods accept an
  `AuthenticatedPrincipal` for public selected image read/preflight flows.
* [x] Unauthorized selected artwork GET and HEAD return `403`, including when
  an `If-None-Match` header is present.
* [x] Authorized selected artwork GET and HEAD behavior remains unchanged for
  original and variant images.
* [x] Matching `If-None-Match` still returns `304 Not Modified` only after app
  service access enforcement.
* [x] Focused server formatting, check, and nextest gates pass or any skipped
  gate is recorded with a concrete reason.

## Verification

* `cargo fmt --all`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks --no-fail-fast`
* `cargo nextest run -p nako-server managed_artwork_variant_routes_resize_selected_artwork_without_locator_or_hash_leaks --no-fail-fast`
* `cargo nextest run -p nako-server catalog --no-fail-fast`
* `git diff --check`
* `python ./.trellis/scripts/task.py validate 06-09-selected-artwork-access-app`

## Definition of Done

* Code is formatted with `cargo fmt --all`.
* `cargo check -p nako-server --tests` passes.
* Focused selected artwork/catalog route tests pass under `cargo nextest`.
* Relevant `.trellis/spec/` guidance is updated to replace the old route-local
  selected artwork access baseline.
* Task context validates, task is archived after commit, and the session is
  recorded in the Trellis journal.

## Technical Approach

Add a managed artwork app-service access helper that loads the selected
artwork record, checks the caller's effective library Browse access for
`selected.library_id`, and returns the selected/artifact pair used by both
preflight and byte reads. Update public selected image methods to require the
principal and reuse that helper before variant validation, so the old route
ordering remains intact.

## Decision (ADR-lite)

**Context**: Public selected artwork image routes still held a route-local
library access check, unlike adjacent catalog/user state surfaces where the app
service owns access before shaping data.

**Decision**: Move the Browse access check into `ManagedArtworkAppService`
selected-image read/preflight methods, keeping HTTP response construction in
`http::catalog`.

**Consequences**: The app service becomes the authority for selected artwork
read eligibility. HTTP remains responsible for HTTP-only cache headers and
request validator parsing. This avoids widening core repository traits and
keeps the refactor scoped to server app/http boundaries.

## Out of Scope

* Metadata manage access.
* Playback, renderer, source, or media-byte access checks.
* Cache header, ETag generation, or conditional request policy redesign.
* Public DTO, API schema, database schema, or migration changes.
* Moving selected artwork HTTP response helpers out of `http::catalog`.

## Technical Notes

* Relevant spec: `.trellis/spec/nako-server/backend/http-api-patterns.md`.
* Relevant spec: `.trellis/spec/nako-server/backend/error-handling.md`.
* Relevant spec: `.trellis/spec/nako-server/backend/quality-guidelines.md`.
* Relevant architecture: `docs/architecture/CONTROL_PLANE.md`.
* Relevant architecture: `docs/architecture/LIBRARY_PIPELINE.md`.
* Main code paths:
  `crates/nako-server/src/http/catalog.rs`,
  `crates/nako-server/src/app/artwork.rs`,
  `crates/nako-server/src/http/access.rs`,
  `crates/nako-server/src/http/tests/catalog.rs`.
