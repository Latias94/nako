# Admin Web Browse Filter UI Patterns

## Scope

This note records the local Admin Web patterns relevant to adding filter, sort,
and search controls to the Media Web items browse page.

## Existing Pattern

- Route-owned pages keep browse/search state in URL search params.
- Route components in `apps/admin-web/src/App.tsx` validate and normalize search
  params, then pass `search` plus `onSearchChange(next)` into route modules.
- Page components render controls with `FilterBar`, `FilterField`, and
  `FilterActions`.
- Filter changes reset `offset` to `0`.
- Pagination changes only update `offset`.
- Reset/clear actions restore default filters and default pagination.
- Active filter counts exclude pagination-only state; pages separately treat
  non-default `limit` or `offset` as a pagination delta.
- Tests set `window.history.pushState(...)`, interact with controls, assert URL
  search params, and assert data-source calls.

## Reference Pages

- `apps/admin-web/src/features/catalog/CatalogBrowsePage.tsx`
  - Provides the closest browse/search baseline.
  - Uses `q`, `facet`, `limit`, and `offset`.
  - Calls `onSearchChange({ q/facet, offset: 0 })` for filter changes.
  - Clears `q`, `facet`, `limit`, and `offset` together.
- `apps/admin-web/src/features/events/EventsPage.tsx`
  - Shows multiple independent filter fields and pagination in one `FilterBar`.
- `apps/admin-web/src/features/storage/StorageStagingPage.tsx`
  - Shows select-driven filters and reset behavior.
- `apps/admin-web/src/features/playback/PlaybackSessionsPage.tsx`
  - Shows route-owned operational filters with safe test assertions.

## Media Web Current State

- `MediaItemsPage` currently renders a browse header, grid, and `MediaPager`.
- `/media/items` currently validates only `limit` and `offset`.
- `MediaSearchPage` already supports `q`, `facet`, `limit`, and `offset` through
  a separate `/media/search` route.
- Media Web read errors must be mapped to static safe copy before rendering.

## Implementation Guidance

- Extend `/media/items` with a route-owned browse search type rather than local
  component-only state.
- Use a `FilterBar` above the grid, aligned with `CatalogBrowsePage`.
- Support controlled native inputs/selects, not a new form library.
- Normalize empty strings to `undefined`.
- Reset `offset` to `0` for any filter, sort, order, or page-size change.
- Keep `MediaPager` as the only next/previous pagination control.
- Keep rendered errors static and safe.

