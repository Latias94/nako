# Media Library Items Browse

## Goal

Expose the Public Client's real library-scoped item browse capability in Media
Web by adding an items browse panel to the existing Media Library detail page.

## Requirements

- Extend `/media/libraries/$libraryId` search state with library item browse
  params:
  - `facet?: string`
  - `sort?: "title" | "release_date" | "date_added" | "last_played"`
  - `order?: "asc" | "desc"`
  - `watch_state?: "watched" | "unwatched" | "in_progress"`
  - `limit: number`
  - `offset: number`
- Add `MediaWebDataSource.listLibraryItems(libraryId, query)` and wire live mode
  to generated `client.listLibraryItems`.
- Render a "Library items" panel on `MediaLibraryDetailPage` above the existing
  "Library sources" panel.
- Reuse the Media item card/grid and the browse filter controls from
  `/media/items` where practical.
- Filter/sort/watch-state/limit changes reset `offset=0`.
- Clear restores default browse state: no `facet`, no `sort`, no `order`, no
  `watch_state`, `limit=20`, `offset=0`.
- Keep the existing Library sources panel visible and paginated by the same
  route pagination for this MVP.
- Preserve safe static error copy for Media Web reads.

## Acceptance Criteria

- [x] `/media/libraries/library-anime?limit=1&offset=0` renders a "Library items"
      panel and calls `listLibraryItems("library-anime", { limit: 1, offset: 0 })`.
- [x] Library item filters update URL search and reset `offset=0`.
- [x] Rich library item browse params are forwarded to live
      `client.listLibraryItems`, not dropped like top-level `/items`.
- [x] `watch_state=any` normalizes to omitted state.
- [x] The existing Library sources panel still renders.
- [x] Unsafe errors and unsafe returned fields are not rendered.
- [x] Focused Media Web route/data-source tests pass.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] `npm run build --prefix apps/admin-web` passes.

## Definition of Done

- Tests cover route URL state, data-source forwarding, fixture behavior, and
  redaction.
- TypeScript check, focused tests, and build pass.
- Task research and context are persisted in Trellis.
- Spec update is considered; update only if this adds a new durable contract not
  covered by the existing Media Web browse spec.

## Technical Approach

- Reuse `MediaItemsBrowseSearch` for the library detail route search shape.
- Extract the Media items filter controls if needed, or keep the helper local if
  reuse stays simple.
- Add `listLibraryItems` to the Media Web data-source interface and lazy proxy.
- In fixture mode, return deterministic `fixtureItems` for matching libraries.
- In live mode, call generated `client.listLibraryItems(libraryId, query)`.
- Add route/data-source tests in `mediaSurface.test.tsx` and
  `mediaDataSource.test.ts`.

## Decision (ADR-lite)

Context: The previous `/media/items` work added rich browse UI, but top-level
`/items` is not a rich browse endpoint yet. The backend and generated Public
Client SDK already support rich browse semantics on `/libraries/{id}/items`.

Decision: Implement the real rich browse path inside the existing Media Library
detail page as a panel, rather than adding a separate route first.

Consequences: The route stays simple and immediately useful. A later task can
split the panel into `/media/libraries/$libraryId/items` if the UI grows beyond
one detail page.

## Out of Scope

- New `/media/libraries/$libraryId/items` route.
- Backend or SDK changes.
- Text search within a library.
- Separate pagination state for sources and items.
- Facet discovery/autocomplete.

## Research References

- `research/library-items-capability.md` - local capability and boundary map.

## Technical Notes

- Relevant specs:
  - `.trellis/spec/admin-web/frontend/index.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/index.md`
