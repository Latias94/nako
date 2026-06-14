# Media Web Recently Added Home Section

## Goal

Make the Media Web home page a clearer content-discovery entry point by presenting the existing first page of media items as a `Recently Added` section. This gives users an immediate browse path alongside `Continue Watching` without adding backend/API scope.

## What I Already Know

* The Media Web home page already loads `listItems({ limit: 8, offset: 0 })` in `apps/admin-web/src/surfaces/media/MediaPages.tsx`.
* The current section title is generic (`Media Items`), so the product intent is unclear.
* Media Web reads go through `MediaWebDataSource` and must not add page-level `fetch`.
* Existing tests assert Media Web redaction: no bearer tokens, browser playback tickets, stream URLs, source paths, fingerprints, or raw internals should appear as text.
* Fixture data already returns two items (`Pilot`, `After the Rain`) and can support this UI without backend changes.

## Requirements

* Rename/structure the Media Web home item section as `Recently Added`.
* Continue using the existing Media Web data-source boundary and `listItems({ limit: 8, offset: 0 })`.
* Render item cards that link to `/media/items/$itemId` using existing card behavior.
* Show a meaningful count in the section header based on the item response page.
* Show an explicit empty state when the recently-added response contains no items and is not loading.
* Preserve existing loading and error states for the item section.
* Preserve redaction boundaries; unsafe tokens, URLs, paths, fingerprints, and source internals must not be rendered.

## Acceptance Criteria

* [x] `/media` in fixture mode shows `Recently Added`, includes item cards, and links cards to item detail routes.
* [x] The home page calls `listItems` with `{ limit: 8, offset: 0 }`.
* [x] An empty item response shows a `No recently added media` empty state.
* [x] A failed item response shows the existing safe item-section error path without replacing `Continue Watching`.
* [x] Tests assert the section does not render known unsafe media internals.

## Definition of Done

* Tests added or updated in `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`.
* `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx` passes.
* `npm run check --prefix apps/admin-web` passes.
* `npm run test --prefix apps/admin-web` passes unless runtime cost or unrelated failures are documented.
* `npm run build --prefix apps/admin-web` passes when page code changes.
* Trellis task validates and is archived after completion.

## Technical Approach

Use the existing home-page item load and `MediaItemGrid` rendering path. Rename the section and add a targeted empty branch inside `MediaItemGrid` or a small caller-owned label prop if needed. Keep this as a frontend-only copy/state/test improvement; no SDK, server, or protocol changes are required.

## Decision (ADR-lite)

**Context**: Nako needs a home content-discovery surface, but the backend contract currently exposes generic item listing, not an explicit recently-added sort contract.

**Decision**: Treat the existing first `listItems` page as the initial `Recently Added` UI slice. Do not add API sort/filter parameters in this task.

**Consequences**: The UI gets a product-shaped discovery entry now while preserving the current Public Client contract. A future backend task can introduce explicit item ordering semantics if product needs require a stronger guarantee.

## Out of Scope

* Backend/Public Client API changes.
* New sorting or filtering semantics.
* Recommendations, personalization, row carousels, posters, or artwork ingestion.
* Media Web route/bundle restructuring.
* Broad `mediaSurface.test.tsx` helper refactors unless required by the new tests.

## Technical Notes

* Relevant files inspected:
  * `apps/admin-web/src/surfaces/media/MediaPages.tsx`
  * `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
  * `apps/admin-web/src/surfaces/media/mediaDataSource.ts`
  * `apps/admin-web/src/surfaces/media/fixtures.ts`
  * `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
* Follow Admin Web route/data rules: no page-level `fetch`, preserve URL-owned state where present, keep sensitive media transport material out of rendered text.
