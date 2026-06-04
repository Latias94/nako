# Research: selected artwork conditional GET

- Query: Find the narrowest conditional HTTP response slice for selected
  artwork image byte routes.
- Scope: internal code/docs only.
- Date: 2026-06-04.

## Findings

### Files inspected

* `crates/nako-server/src/http/catalog.rs` - selected artwork GET and HEAD
  handlers call `selected_image_response`, which owns response header assembly.
* `crates/nako-server/src/app/artwork/variant.rs` - current selected artwork
  ETags are generated after the selected artwork artifact and variant have been
  resolved.
* `crates/nako-server/src/http/tests/addons.rs` - existing selected artwork
  route tests already exercise original GET/HEAD, variant GET/HEAD, ETags,
  cache-control, content length, and redaction.
* `.trellis/spec/nako-server/backend/http-api-patterns.md` - selected artwork
  private cache-control is now route-local and conditional GET is explicitly a
  follow-on.
* `docs/architecture/CONTROL_PLANE.md` - selected artwork image cache baseline
  exists; conditional GET remains a cache-contract follow-on.

### Current behavior

Selected artwork image routes return 200 with body for GET, even when a client
already has the current ETag and sends `If-None-Match`.

### Candidate approaches

#### Approach A: route-local post-read exact match (recommended)

Read/derive the current selected artwork image as today, compare the quoted ETag
against the request's `If-None-Match`, and return 304 when it matches exactly.

Pros:

* Very small HTTP-only slice.
* Preserves auth, access checks, ETag generation, and app-service behavior.
* Avoids public DTO, schema, generated contract, and storage changes.

Cons:

* Does not avoid artifact reads or image resizing before the 304 decision.
* Exact matching does not implement every HTTP validator form.

#### Approach B: metadata-only app-service ETag preflight

Add an app-service method that resolves selected artwork/artifact metadata and
variant identity, computes the ETag without reading bytes, and only reads image
bytes on a miss.

Pros:

* Saves backend work when the client has a current ETag.
* More useful for large images and variants.

Cons:

* Requires a new service seam and careful duplication avoidance for variant
  ETag identity.
* Larger than needed for a first conditional-response contract.

#### Approach C: broad HTTP validator support

Implement wildcard, weak validators, comma-separated ETag sets, `If-Match`,
`Last-Modified`, and 412 handling.

Pros:

* More complete HTTP cache behavior.

Cons:

* Much larger protocol surface.
* Not needed to unblock common browser/app `If-None-Match` revalidation.

## Recommendation

Use Approach A. Keep the first slice route-local and test exact matches for
original and variant selected artwork URLs. Leave metadata-only preflight and
broader validator parsing to dedicated follow-ons.

## Guardrails

* Auth and library access checks must run before any 304 response.
* 304 must not expose storage URI, local path, provider URL, raw content hash,
  token, or source locator material.
* 304 must not change HLS, Direct Play, Remux, Admin, or JSON route behavior.
