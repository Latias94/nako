# Selected Artwork Weak And Wildcard ETag Validators

## Goal

Complete the next selected artwork cache-contract slice by allowing authenticated
selected artwork byte routes to honor weak and wildcard `If-None-Match`
validators while preserving the existing safe selected-artwork ETag authority,
auth/access checks, private cache policy, and metadata-derived preflight path.

## Requirements

- Extend selected artwork `If-None-Match` matching so these request forms return
  `304 Not Modified` when the current selected artwork ETag exists:
  - exact quoted strong tag already supported today;
  - weak validator form `W/"etag"` for the current safe ETag;
  - comma-separated validator lists that contain the current safe ETag in either
    strong or weak form;
  - wildcard `*`.
- Apply the same matching behavior to both normal byte response matching and
  metadata-derived selected artwork ETag preflight.
- Preserve the selected artwork response contract:
  - auth and library access checks run before any 304 response;
  - 304 responses include `ETag` and `Cache-Control: private, max-age=86400`;
  - 304 responses have an empty body;
  - non-matching, malformed, or unsupported validator input keeps the existing
    `200` GET/HEAD behavior.
- Keep variant ETags distinct from original ETags. A request for a variant must
  not match the original ETag unless `If-None-Match` is `*`.
- Update local server/API documentation and Trellis spec language so the new
  weak/wildcard contract is no longer listed as a follow-on.

## Acceptance Criteria

- [x] Original selected artwork GET returns 304 for `If-None-Match: W/"etag"`.
- [x] Original selected artwork GET returns 304 when a validator list includes
      the current ETag.
- [x] Original selected artwork GET returns 304 for `If-None-Match: *`.
- [x] Resized selected artwork variant GET returns 304 for weak and wildcard
      validators against the variant route.
- [x] Variant route still returns 200 when `If-None-Match` contains only the
      original image ETag.
- [x] Metadata-derived preflight still returns 304 after the backing artifact
      bytes are removed when weak/wildcard validators match.
- [x] Existing forbidden, missing, invalid-variant, HEAD, content headers,
      private cache-control, and locator/hash redaction behavior is unchanged.
- [x] Focused `nako-server` nextest gate passes.

## Definition Of Done

- Code and tests are formatted with repo Rust formatting.
- Focused tests covering selected artwork conditional response behavior pass.
- `cargo check -p nako-server --tests` passes unless a pre-existing broader
  workspace issue blocks it and is recorded.
- `git diff --check` passes.
- Documentation/spec updates are included when behavior changes.
- Task is validated, completed, archived, and committed with a Conventional
  Commit message.

## Technical Approach

Keep matching route-local in `crates/nako-server/src/http/catalog.rs` where the
selected artwork route already owns safe ETag header authoring. Replace strict
`HeaderValue` equality with a small parser that:

- rejects non-UTF-8 header values by returning no match;
- trims comma-separated entity-tag candidates;
- recognizes `*` as a match when the route has a current selected artwork ETag;
- recognizes both exact quoted and weak quoted forms for the current safe ETag;
- does not compare against raw backend/source ETags or artifact content hashes.

Use the existing selected artwork route test in
`crates/nako-server/src/http/tests/addons.rs` as the focused integration test
surface, because it already covers original, variant, HEAD, preflight, access
denial, redaction, and invalid variant behavior.

## Decision (ADR-lite)

Context: The previous selected artwork cache slice intentionally shipped only
exact `If-None-Match` matching and left weak/wildcard validators as a follow-on.
The metadata-derived preflight slice is now shipped, so weak/wildcard matching
is the smallest useful remaining protocol-compliance improvement.

Decision: Implement weak, validator-list, and wildcard matching inside the
selected artwork HTTP helper without changing ETag generation, storage state,
public DTOs, or cross-route cache semantics.

Consequences: The HTTP behavior becomes more compatible with standard client
caches while keeping the same private selected artwork cache boundary. Broader
cache invalidation, derivatives/placeholders, immutable/shared-cache policy, and
CDN semantics remain separate tasks.

## Out Of Scope

- New ETag generation, content-hash exposure, or artifact/source ETag exposure.
- `Last-Modified`, `If-Modified-Since`, `If-Match`, or range/cache validator
  behavior for media bytes.
- HLS, Direct Play, Remux, Admin JSON, or public catalog JSON cache changes.
- Selected artwork invalidation, persisted derivatives, placeholder/Blurhash
  generation, CDN/shared-cache policy, or storage schema changes.

## Technical Notes

- Current helper:
  `crates/nako-server/src/http/catalog.rs::selected_image_etag_matches`.
- Current tests:
  `crates/nako-server/src/http/tests/addons.rs::public_catalog_and_image_routes_serve_selected_artwork_without_locator_leaks`.
- Before this task, the server spec said weak/wildcard validators were out of
  scope:
  `.trellis/spec/nako-server/backend/http-api-patterns.md`.
- Before this task, active architecture maps listed weak/wildcard validator
  support as the selected artwork cache follow-on:
  `docs/architecture/CONTROL_PLANE.md` and
  `docs/architecture/LIBRARY_PIPELINE.md`.
