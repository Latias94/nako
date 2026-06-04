# Selected Artwork ETag Preflight Map Reconciliation

## Goal

Update architecture maps after metadata-derived selected artwork ETag preflight
shipped in the 10-hour campaign, so future planning does not reopen a completed
cache-contract slice.

## Requirements

- Mark metadata-only selected artwork ETag preflight as shipped in
  `docs/architecture/CONTROL_PLANE.md`.
- Mark the same shipped boundary in `docs/architecture/LIBRARY_PIPELINE.md`.
- Preserve remaining artwork delivery cache follow-ons: derivative generation,
  placeholders/Blurhash, WebP/size presets, weak/wildcard validator support,
  immutable/shared-cache semantics, CDN behavior, and selected-artwork
  invalidation.
- Link the campaign implementation evidence:
  `.trellis/tasks/archive/2026-06/06-04-10-hour-media-server-architecture-campaign/implementation/lane-c-artwork-cache.md`.
- Do not change Rust code, HTTP route behavior, generated contracts, schema, or
  public API shape.

## Acceptance Criteria

- [x] `CONTROL_PLANE.md` no longer lists metadata-only selected artwork ETag
  preflight as a follow-on.
- [x] `LIBRARY_PIPELINE.md` no longer lists metadata-only selected artwork ETag
  preflight in the open artwork delivery cache scope.
- [x] Remaining cache follow-ons stay explicit and do not imply shared/public
  cache semantics are shipped.
- [x] No implementation files change.

## Definition of Done

- `git diff --check`
- `python ./.trellis/scripts/task.py validate 06-05-selected-artwork-etag-preflight-map-reconciliation`
- Focused grep confirms stale metadata-only preflight wording is gone from the
  active architecture maps.

## Technical Approach

Use the existing code-spec and campaign implementation evidence as source of
truth. The current server spec already lists `selected_image_preflight_response`
as the metadata-derived exact-match 304 short-circuit after auth and library
access checks. The architecture maps should match that shipped state.

## Decision (ADR-lite)

**Context**: Code and spec include metadata-derived selected artwork preflight,
but active architecture maps still list it as a future follow-on.

**Decision**: Reconcile only the architecture maps, leaving the remaining cache
work as future follow-ons.

**Consequences**: Future task selection can focus on still-open artwork cache
work instead of repeating completed preflight behavior.

## Out of Scope

- No weak or wildcard validator parsing.
- No `Last-Modified`, immutable headers, shared-cache/CDN behavior, or cache
  invalidation policy.
- No derivative store, WebP/size preset, placeholder, or Blurhash work.
- No route, DTO, schema, generated SDK, or Rust implementation change.

## Research References

- [`research/selected-artwork-etag-preflight-evidence.md`](research/selected-artwork-etag-preflight-evidence.md)
  - shipped evidence and remaining cache-contract boundary.
