# Research: selected artwork cache-control headers

- Query: Find the narrowest HTTP cache-contract slice for selected artwork image
  byte responses.
- Scope: internal code/docs only.
- Date: 2026-06-04.

## Findings

### Files inspected

* `docs/architecture/CONTROL_PLANE.md` - HTTP cache/ETag contracts are partial;
  image/artwork cache semantics remain follow-ons. HLS `no-store` is already
  documented as a narrow session-artifact baseline.
* `docs/architecture/LIBRARY_PIPELINE.md` - artwork artifact lifecycle is
  shipped, while delivery cache placeholders and broader derivative policy
  remain next-lane work.
* `docs/workstreams/managed-artwork-public-serving-selection/DESIGN.md` -
  selected artwork byte routes are authenticated first-party routes; safe ETags
  are part of the public image reference model, while range requests,
  thumbnails, variants, and cache eviction were split.
* `crates/nako-server/src/http/catalog.rs` - `selected_image_response` owns the
  response header assembly for both GET and HEAD selected artwork image routes.
* `crates/nako-server/src/app/artwork/variant.rs` - safe ETags are generated
  from selected artwork identity, artifact identity, artifact update time, and
  variant key. Raw content hashes are not returned by the byte route.
* `crates/nako-server/src/http/tests/addons.rs` - existing tests cover selected
  artwork GET, HEAD, resized variants, ETag, content length, and redaction.
* `.trellis/spec/nako-server/backend/http-api-patterns.md` - HLS artifacts use
  an HLS-only `Cache-Control: no-store` helper; broader cache behavior must not
  be changed by accident.

### Current behavior

Selected artwork image GET/HEAD responses currently include:

* `Content-Type`
* `Content-Length`
* `ETag` when available

They do not include an explicit `Cache-Control` header.

### Candidate approaches

#### Approach A: private max-age baseline (recommended)

Set `Cache-Control: private, max-age=86400` for selected artwork byte
responses.

Pros:

* Small route-local change.
* Suitable for authenticated library-scoped image routes.
* Lets browser/app caches reuse images without enabling shared proxy reuse.
* Does not require conditional GET, schema, public DTOs, or artifact lifecycle
  changes.

Cons:

* Clients may keep stale artwork for up to one day after reselection unless the
  ETag change is checked or the client refetches.
* Does not solve CDN/shared-cache or invalidation strategy.

#### Approach B: no-store baseline

Set `Cache-Control: no-store` like HLS session artifacts.

Pros:

* Most conservative.
* Avoids stale selected artwork entirely.

Cons:

* Defeats the purpose of artwork delivery cache contracts.
* Treats long-lived selected artwork like session-scoped playback artifacts,
  which conflicts with the product goal of fast catalog artwork.

#### Approach C: conditional GET and 304 now

Parse `If-None-Match`, compare to the derived selected image ETag, and return
`304 Not Modified`.

Pros:

* More complete HTTP cache behavior.
* Reduces byte transfer after revalidation.

Cons:

* Requires request-header handling and 304 header/body behavior across GET and
  HEAD.
* Larger behavioral surface than needed for the first selected-artwork cache
  contract.

## Recommendation

Use Approach A for this slice. Keep the helper selected-artwork-specific and
test the existing GET/HEAD and variant paths. Leave Approach C as a follow-on
once cache invalidation and conditional-response behavior are specified.

## Guardrails

* Do not change HLS `no-store`, Direct Play, Remux, Admin, or JSON route cache
  behavior.
* Do not expose raw content hashes, storage URIs, local paths, source URLs,
  provider payloads, or tokens.
* Do not add generated contract changes; the byte route headers are server HTTP
  behavior, not a DTO shape change.
