# Selected Artwork Conditional GET First Slice

## Goal

Add a narrow HTTP conditional-response contract for authenticated selected
artwork image byte routes so clients that already have the current safe ETag can
receive `304 Not Modified` instead of another image body.

## What I Already Know

* `GET /images/{image_id}` and `HEAD /images/{image_id}` now return
  `Cache-Control: private, max-age=86400`.
* Selected artwork image responses already have safe public ETags generated from
  selected artwork identity, artifact identity, artifact update time, and image
  variant key.
* `selected_image_response` is the shared response assembly point for original
  GET, original HEAD, variant GET, and variant HEAD selected artwork routes.
* The selected artwork ETag is currently available only after the app service
  has loaded the selected artwork, artifact, and image bytes. This slice should
  not add a new metadata-only ETag service.
* `.trellis/spec/nako-server/backend/http-api-patterns.md` now records the
  selected artwork cache-control baseline and leaves conditional GET as a
  follow-on.

## Assumptions

* A first conditional GET slice may compare `If-None-Match` after deriving the
  current image response. This reduces response body transfer but does not yet
  avoid artifact reads or variant resizing.
* The route must keep auth and library access checks before returning 304.
* Only exact strong ETag matches are required in this slice. Wildcard,
  weak-validator, comma-list, `If-Match`, and `Last-Modified` behavior are
  follow-ons unless existing Axum/header behavior makes exact comma-list support
  trivial and safe.

## Requirements

* Accept request headers in selected artwork GET and HEAD route handlers.
* If `If-None-Match` exactly matches the current selected artwork ETag, return
  `304 Not Modified`.
* A 304 response must include the current `ETag` and
  `Cache-Control: private, max-age=86400`.
* Preserve existing GET/HEAD behavior when the header is missing, malformed, or
  does not match.
* Preserve selected artwork auth, library access, image variant query handling,
  content headers, ETag generation, and redaction behavior.
* Add focused HTTP tests for matching and non-matching `If-None-Match` on
  original and variant selected artwork URLs.

## Acceptance Criteria

* [x] Matching `If-None-Match` on original selected artwork GET returns 304,
  includes ETag/cache headers, and has no body.
* [x] Matching `If-None-Match` on a selected artwork variant GET returns 304
  only for that variant's ETag.
* [x] Non-matching `If-None-Match` preserves normal 200 image response behavior.
* [x] HEAD responses preserve the existing no-body behavior and headers.
* [x] No DTO, generated contract, schema, HLS, Direct Play, Remux, Admin, or
  selected artwork ETag identity changes are introduced.
* [x] Focused server checks pass and evidence is recorded.

## Definition Of Done

* Code is committed with a Conventional Commit message.
* Validation evidence is recorded in this task.
* Relevant spec/architecture docs are updated if the conditional-response
  contract becomes a reusable convention.
* The parent long-horizon task records this child outcome.
* The task is archived and the developer journal is updated.

## Technical Approach

Thread `HeaderMap` into `get_image` and `head_image`, then pass an extracted
`If-None-Match` value to `selected_image_response`. Keep matching logic local to
the public catalog route module. Reuse the quoted ETag header string already
authored for selected artwork responses, so the comparison and returned ETag
cannot drift.

This slice should return 304 after the app service has derived the current
`ManagedArtworkImageBytes`. A future optimization can split metadata-only ETag
derivation before reading or resizing bytes.

## Decision (ADR-lite)

Context: Selected artwork private cache headers and safe ETags now exist, but
clients still receive the full image body even when they revalidate with the
current ETag.

Decision: Implement route-local exact `If-None-Match` matching for selected
artwork GET/HEAD responses and return 304 with ETag/cache headers when matched.

Consequences: Client revalidation semantics become correct for selected artwork
without changing public DTOs or storage services. The first slice does not yet
avoid backend artifact reads or variant derivation work; that performance
optimization stays separate.

## Out Of Scope

* No metadata-only ETag preflight or app-service split.
* No wildcard, weak ETag, `Last-Modified`, `If-Match`, or immutable/public
  shared-cache behavior.
* No generated contract, OpenAPI, SDK, schema, or DTO changes.
* No changes to selected artwork ETag identity or image variant generation.
* No HLS, Direct Play, Remux, or Admin cache behavior changes.

## Verification Plan

* `cargo fmt --all -- --check`
* `cargo check -p nako-server --tests`
* `cargo nextest run -p nako-server public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks --no-fail-fast`
* `cargo nextest run -p nako-server managed_artwork_variant_routes_resize_selected_artwork_without_locator_or_hash_leaks --no-fail-fast`
* `git diff --check`

## Verification Evidence

* PASS: `cargo fmt --all -- --check`
* PASS: `cargo check -p nako-server --tests`
* PASS: `cargo nextest run -p nako-server public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks --no-fail-fast`
* PASS: `cargo nextest run -p nako-server managed_artwork_variant_routes_resize_selected_artwork_without_locator_or_hash_leaks --no-fail-fast`
* PASS: `git diff --check`
* PASS: `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-selected-artwork-conditional-get-first-slice`

## Spec Update

Updated `.trellis/spec/nako-server/backend/http-api-patterns.md` so selected
artwork image cache-control now includes exact `If-None-Match` / `304 Not
Modified` route behavior, test requirements, and wrong/correct examples.
Updated `docs/architecture/CONTROL_PLANE.md` and
`docs/architecture/LIBRARY_PIPELINE.md` to mark this first conditional-response
baseline shipped while keeping metadata-only ETag preflight and broader cache
semantics as follow-ons.
