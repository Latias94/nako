# Selected Artwork Cache-Control Headers First Slice

## Goal

Give authenticated selected artwork image byte responses an explicit HTTP cache
contract so clients can safely reuse already-fetched artwork without changing
public DTOs, storage schemas, or image variant generation behavior.

## What I Already Know

* `docs/architecture/CONTROL_PLANE.md` lists HTTP cache/ETag contracts as a
  partial control-plane capability and calls out image/artwork cache semantics
  as follow-on work.
* `docs/architecture/LIBRARY_PIPELINE.md` keeps artwork delivery cache
  placeholders and broader derivative policy as an artwork follow-on.
* `GET /images/{image_id}` and `HEAD /images/{image_id}` are authenticated
  Public Client selected artwork byte routes in
  `crates/nako-server/src/http/catalog.rs`.
* Selected artwork image responses already include `Content-Type`,
  `Content-Length`, and safe public ETags derived from selected-artwork identity,
  artifact identity, artifact update time, and variant dimensions.
* HLS session artifacts now use `Cache-Control: no-store`, but selected artwork
  is a different route class: it is authenticated and selected-artwork scoped,
  not playback-ticket/session scoped.

## Assumptions

* Selected artwork byte responses should be cacheable only in private client
  caches because access depends on user/library permissions.
* This slice should set a conservative private cache baseline and leave shared
  caches, CDN semantics, conditional GET, immutable headers, and invalidation
  policy to later cache-contract tasks.
* Existing ETag generation is safe to keep, but this task should not change ETag
  identity or expose raw content hashes, storage URIs, local paths, or provider
  URLs.

## Requirements

* Add a selected-artwork-only HTTP cache helper for image byte responses.
* Apply the helper to `GET /images/{image_id}` and `HEAD /images/{image_id}`.
* Use a private cache policy appropriate for authenticated selected artwork,
  e.g. `Cache-Control: private, max-age=86400`.
* Preserve existing content type, content length, body/no-body behavior, ETag,
  auth, library access checks, and variant query behavior.
* Add focused HTTP route assertions for original GET, original HEAD, variant
  GET, and variant HEAD responses.
* Do not modify public/Admin DTO shapes, generated contracts, storage schemas,
  selected artwork persistence, artifact stores, or image variant generation.

## Acceptance Criteria

* [x] Original selected artwork GET responses include the selected artwork cache
  policy plus existing content headers and ETag.
* [x] Original selected artwork HEAD responses include the same cache policy and
  no body.
* [x] Variant selected artwork GET/HEAD responses include the same cache policy
  while preserving variant-specific ETags and content lengths.
* [x] No HLS, Direct Play, Remux, Admin, schema, or generated API contract
  behavior changes are introduced.
* [x] Focused server checks pass and evidence is recorded.

## Definition Of Done

* Code is committed with a Conventional Commit message.
* Validation evidence is recorded in this task.
* Relevant architecture/spec docs are updated to record the selected artwork
  cache baseline.
* The parent long-horizon task records this child outcome.
* The task is archived and the developer journal is updated.

## Technical Approach

Add a small helper near `selected_image_response` in the public catalog HTTP
module. The helper should be specific to selected artwork image bytes and should
insert `Cache-Control: private, max-age=86400`. Calling the helper inside
`selected_image_response` covers GET and HEAD because that function already owns
the response header assembly for both paths.

Update the existing selected artwork route test in
`crates/nako-server/src/http/tests/addons.rs` because it already exercises
original bytes, HEAD behavior, resized variants, ETags, content length, and
redaction. Keep the test focused on header behavior rather than adding new
fixtures.

## Decision (ADR-lite)

Context: Selected artwork routes already have safe ETags but no explicit
`Cache-Control` header. HLS artifacts recently gained a `no-store` baseline, but
selected artwork is long-lived client-visible artwork rather than
session-scoped playback data.

Decision: Set authenticated selected artwork bytes to a private client-cache
baseline: `Cache-Control: private, max-age=86400`.

Consequences: Browsers and app HTTP caches can reuse selected artwork for one
day within a private cache, while shared proxies are told not to store it for
other users. Conditional GET, immutable cache keys, CDN behavior, and selected
artwork invalidation remain separate follow-ons.

## Out Of Scope

* No `If-None-Match` / `304 Not Modified` handling.
* No `Last-Modified`, immutable, or public shared-cache policy.
* No derivative cache persistence or thumbnail-generation changes.
* No changes to selected artwork URLs, ETags, DTOs, OpenAPI, SDK, or schema.
* No changes to HLS, Direct Play, Remux, or Admin response cache behavior.

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
* PASS: `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-selected-artwork-cache-control-headers-first-slice`
* NOTE: An initial command using the helper name
  `assert_selected_artwork_variant_serving_without_locator_or_hash_leaks`
  returned `0 tests run`; the verification plan was corrected to the two real
  route test names above.

## Spec Update

Updated `.trellis/spec/nako-server/backend/http-api-patterns.md` with the
selected artwork image cache-control contract, including signatures, route-only
scope, validation matrix, good/base/bad cases, tests, and wrong/correct
examples. Updated `docs/architecture/CONTROL_PLANE.md` and
`docs/architecture/LIBRARY_PIPELINE.md` to record the selected artwork private
cache baseline and remaining follow-ons.
