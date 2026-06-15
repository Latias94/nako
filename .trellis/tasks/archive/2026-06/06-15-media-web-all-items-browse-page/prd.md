# Media Web All Items Browse Page

## Goal

Add a full browse path for Media Web items so the home page's `Recently Added` section can lead into a complete paginated list of media items.

## What I Already Know

* `apps/admin-web/src/surfaces/media/MediaPages.tsx` already renders a `Recently Added` home section from `listItems({ limit: 8, offset: 0 })`.
* `MediaItemCard`, `MediaItemGrid`, and `MediaPager` already exist and can support an item browse page without new backend contracts.
* `apps/admin-web/src/App.tsx` already owns Media Web route search validation and has a route slot for `/media/items/$itemId`, but not `/media/items`.
* Media Web read sections must keep unsafe source/backend error strings out of rendered text.
* The app already uses `MediaPages.tsx` and lazy route modules for Media Web pages.

## Requirements

* Add a Media Web browse page at `/media/items`.
* The browse page should load media items through the existing `listItems` data-source method.
* The browse page should be paginated with the existing `limit` and `offset` route search behavior.
* Add a `View all` entry from the home `Recently Added` section to the browse page.
* Reuse existing Media Web item card rendering.
* Preserve safe loading, empty, and error states.
* Keep unsafe token/path/fingerprint/backend details out of rendered text.

## Acceptance Criteria

* [ ] `/media/items` renders a paginated item browse page in fixture mode.
* [ ] The browse page calls `listItems` with route search values.
* [ ] Home `Recently Added` includes a `View all` link to `/media/items`.
* [ ] Empty and error states render safe copy.
* [ ] Tests cover route search, item links, pagination, and redaction.

## Definition of Done

* Tests added or updated in `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`.
* `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx` passes.
* `npm run check --prefix apps/admin-web` passes.
* `npm run test --prefix apps/admin-web` passes.
* `npm run build --prefix apps/admin-web` passes.
* Trellis task validates, is completed, and gets archived.

## Technical Approach

Add a dedicated `MediaItemsPage` in `MediaPages.tsx`, wire a new lazy route module from `App.tsx`, and keep the page thin by reusing the existing item card/grid/pager primitives. Extract a small safe `listItems` loader helper if needed so home and browse pages share the same redaction-safe error handling.

## Decision (ADR-lite)

**Context**: The home page already exposes a recent item subset, but users need a way to browse beyond the first 8 items.

**Decision**: Add a lightweight paginated browse page over the existing `listItems` contract instead of inventing new sorting/filtering behavior.

**Consequences**: Users get a complete browse path now, and the backend contract remains unchanged. Future work can add richer browse filters or ordering without breaking this slice.

## Out of Scope

* Backend/Public Client API changes.
* Search, filters, or sorting beyond `limit` / `offset`.
* Navigation shell changes unless needed for route wiring.
* Artwork/poster layout work.

## Technical Notes

* Relevant files inspected:
  * `apps/admin-web/src/App.tsx`
  * `apps/admin-web/src/surfaces/media/MediaPages.tsx`
  * `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
  * `apps/admin-web/src/surfaces/media/mediaDataSource.ts`
  * `apps/admin-web/src/routes/MediaHomeRouteModule.tsx`
  * `apps/admin-web/src/routes/MediaLibrariesRouteModule.tsx`
  * `apps/admin-web/src/routes/MediaSearchRouteModule.tsx`
* Existing Media Web route search types already include `MediaPageSearch`.
