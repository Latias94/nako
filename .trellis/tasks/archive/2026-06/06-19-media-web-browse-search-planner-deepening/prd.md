# Media Web Browse/Search Planner Deepening

## Goal

Make the Media Web browse flow a deep module by pulling route-owned search state, browse planning, and Public Client query forwarding behind a smaller seam. The target is a self-hosted media browsing experience that can combine search, filters, sort, watch state, and pagination without scattering the rules across page components.

## Requirements

* `/media/items` owns browse state in the URL.
* `q`, `facet`, `sort`, `order`, `watch_state`, `limit`, and `offset` remain route-owned search fields.
* `watch_state=any` normalizes to an omitted value.
* Filter, sort, watch-state, and page-size changes reset `offset` to `0`.
* Reset restores default browse state and default pagination.
* Media Web browse/search pages use a smaller browse planner seam instead of carrying query rules in page code.
* Live Media Web forwarding stays compatible with the current top-level Public Client contract.
* Route tests prove URL normalization, search/planner forwarding, and redaction-safe rendering.

## Acceptance Criteria

* [ ] `/media/items` keeps working with pagination-only state.
* [ ] Changing filter/sort/watch-state controls updates the URL and resets `offset` to `0`.
* [ ] Reset clears browse state back to defaults.
* [ ] `watch_state=any` is treated as default state.
* [ ] `q` routes through the text-search path and no-`q` routes through the browse path.
* [ ] Live browse forwarding does not overclaim unsupported top-level `/items` semantics.
* [ ] Media Web tests pass for URL state, forwarding, and safe rendering.
* [ ] `npm run check --prefix apps/admin-web` passes.
* [ ] `npm run build --prefix apps/admin-web` passes.

## Definition of Done

* Route search normalization is centralized.
* Browse/search forwarding is owned by a smaller module seam.
* Tests cover the route behavior that product users will feel.
* No backend contract changes are assumed unless a real gap is found during implementation.

## Technical Approach

* Keep route search validation and normalization in `apps/admin-web/src/App.tsx`.
* Move browse/search query composition into the media surface modules instead of the page body.
* Keep `MediaCore.ts` as the shared type/model home for Media Web search shapes.
* Keep `mediaDataSource.ts` as the live/fixture adapter seam.
* Use `MediaPages.tsx` for rendering, not for query policy.
* Keep live top-level `/items` forwarding compatible with the current contract, and route text search through `searchItems`.

## Decision (ADR-lite)

**Context**: Media Web already has browse and search routes, but the current shape still leaves query policy spread across route validation, page components, and data-source forwarding. That makes sort/filter/watch-state evolution harder to reason about than it needs to be.

**Decision**: Deepen the Media Web browse/search planner now, but do not expand backend scope in the same task. Keep the browser-facing semantics explicit and testable while preserving current Public Client compatibility.

**Consequences**: The frontend gets a smaller seam and clearer locality for browse rules. Richer backend browse semantics can still land later without forcing this task to rework the server contract.

## Out of Scope

* Backend Public Client changes.
* New library-scoped browse route.
* Facet discovery or autocomplete.
* Persisted user filter presets.
* Playback changes unrelated to browse/search.

## Technical Notes

* Relevant specs:
  * `.trellis/spec/admin-web/frontend/index.md`
  * `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  * `.trellis/spec/guides/index.md`
* Research references:
  * `.trellis/tasks/archive/2026-06/06-15-media-web-items-browse-filter/research/ui-patterns.md`
  * `.trellis/tasks/archive/2026-06/06-15-media-web-items-browse-filter/research/media-capabilities.md`
* Code to inspect:
  * `apps/admin-web/src/App.tsx`
  * `apps/admin-web/src/features/catalog/CatalogBrowsePage.tsx`
  * `apps/admin-web/src/surfaces/media/MediaCore.ts`
  * `apps/admin-web/src/surfaces/media/MediaPages.tsx`
  * `apps/admin-web/src/surfaces/media/mediaDataSource.ts`
  * `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
  * `apps/admin-web/src/App.test.tsx`
