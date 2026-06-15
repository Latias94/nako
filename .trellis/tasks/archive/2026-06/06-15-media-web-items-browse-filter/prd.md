# Media Web Items Browse Filter

## Goal

Make `/media/items` a useful browse surface instead of a pagination-only grid by
adding URL-owned filter, search, and sort controls that follow the Admin Web
filter pattern and reflect the media stack's current API capabilities.

## Requirements

- Add route-owned browse state for `/media/items`:
  - `q?: string`
  - `facet?: string`
  - `sort?: "title" | "release_date" | "date_added" | "last_played"`
  - `order?: "asc" | "desc"`
  - `watch_state?: "watched" | "unwatched" | "in_progress"`; UI `Any` is the
    default and normalizes to an omitted URL param.
  - `limit: number`
  - `offset: number`
- Render a `FilterBar` on `MediaItemsPage` with controls for search, facet, sort,
  order, watch state, limit, and reset.
- Keep pagination with the existing `MediaPager`.
- Reset `offset` to `0` when any search, filter, sort, order, watch-state, or
  page-size value changes.
- Reset clears all non-default browse values and restores `limit=20` and
  `offset=0`.
- Prefer the currently available Public Client path:
  - When `q` is present, use `searchItems` with `q`, `facet`, and pagination.
  - When `q` is absent, use `listItems` with the full browse query type, but the
    live implementation may only forward fields supported by the current
    `/items` contract.
- Preserve static safe error copy for Media Web reads. Do not render backend,
  source, token, path, fingerprint, or raw fixture error strings.
- Keep fixture mode deterministic and able to accept the richer query object
  without throwing.

## Acceptance Criteria

- [ ] `/media/items?limit=1&offset=1` still renders and paginates as before.
- [ ] Changing a filter updates the URL and resets `offset=0`.
- [ ] Reset clears `q`, `facet`, `sort`, `order`, and `watch_state`, and restores
      default pagination.
- [ ] A search query on `/media/items` calls `searchItems` with `q`, `facet`,
      `limit`, and `offset`.
- [ ] A browse query without `q` calls `listItems` with the normalized browse
      search object.
- [ ] Unsafe read errors or unsafe returned fields are not rendered.
- [ ] Focused Media Web tests pass.
- [ ] `npm run check --prefix apps/admin-web` passes.
- [ ] `npm run build --prefix apps/admin-web` passes.

## Definition of Done

- Tests added or updated for URL search behavior, data-source calls, reset
  behavior, and safe error rendering.
- TypeScript check and focused tests pass.
- Implementation follows Admin Web route/filter patterns.
- No backend API behavior is implied beyond current contracts unless explicitly
  implemented.

## Technical Approach

- Introduce a Media items browse search type in `MediaCore.ts`.
- Change `/media/items` validation in `App.tsx` to normalize the richer browse
  params while leaving `/media/libraries` and library detail pagination-only.
- Update `MediaItemsRouteModule` and `MediaItemsPage` props to use the richer
  search type.
- Update `MediaWebDataSource.listItems` to accept the richer query shape.
- Keep live `listItems` behavior compatible with the current generated SDK. If
  the SDK only accepts `PageQuery`, forward pagination only for live browse and
  rely on `searchItems` when `q` is present.
- Add tests in `mediaSurface.test.tsx`.

## Decision (ADR-lite)

Context: Admin Web already has URL-owned filter/search patterns, and Media Web
already has `/media/search` for `q`/`facet`. The Public Client has richer
library-scoped item browse (`sort`, `order`, `facet`, `watch_state`), but global
`/items` is currently pagination-only in the server and generated SDK.

Decision: Implement the `/media/items` UX and route state now, route query search
through `searchItems`, and keep live global browse compatible with the current
`/items` contract. This creates a front-end extension point without pretending
the backend supports global sort/filter semantics it does not yet implement.

Consequences: Sort/watch-state controls are represented in URL state and fixture
calls, but full live semantics need a later backend/Public Client slice for
global `/items` or a library-scoped browse entry.

## Out of Scope

- Backend changes to `/items`.
- Public Client generated SDK changes.
- New library-scoped browse route.
- Facet discovery/autocomplete.
- Persisted user filter presets.

## Research References

- `research/ui-patterns.md` - Admin Web URL-owned filter and reset patterns.
- `research/media-capabilities.md` - Media Web and Public Client browse/search
  capability map.

## Technical Notes

- Relevant specs:
  - `.trellis/spec/admin-web/frontend/index.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/index.md`
- Relevant code:
  - `apps/admin-web/src/App.tsx`
  - `apps/admin-web/src/surfaces/media/MediaCore.ts`
  - `apps/admin-web/src/surfaces/media/MediaPages.tsx`
  - `apps/admin-web/src/surfaces/media/mediaDataSource.ts`
  - `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
