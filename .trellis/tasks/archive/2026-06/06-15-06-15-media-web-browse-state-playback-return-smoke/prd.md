# Media Web Browse State Playback Return Smoke

## Goal

Prove that Media Web users can browse with combined item state, open an item, start playback, write progress, and return to the original browse state without losing route-owned filters, sorting, pagination, or playback continuity.

## What I Already Know

- Recent Media Web coverage added a route smoke from Library detail `Library items` to item detail, Watch, progress, Resume, watched state, and Continue Watching refresh.
- Existing route tests separately cover `/media/items` URL-owned pagination, filters, sort, order, watch-state, reset behavior, and `/media/search` URL-owned query state.
- Existing route tests separately cover Library detail item filters and rich browse query forwarding.
- Admin Web frontend spec requires route-owned search params, native controls, safe static error copy, and no rendered playback internals.
- Media Web read/player surfaces must not render raw stream URLs, ticket tokens, bearer tokens, source locators, raw paths, fingerprints, or backend internals.

## Assumptions

- This is an Admin Web / Media Web validation slice, not a backend API, SDK, or product UI redesign task.
- The MVP should add focused route-level smoke coverage and only patch UI behavior if the smoke exposes a real missing affordance.
- Browser history behavior is the right first proof for browse-state return because existing links do not currently carry an explicit `return_to` parameter.

## Open Questions

- None.

## Requirements

- Add a route-level Media Web smoke that starts from `/media/items` with a combined browse state: facet, sort, order, watch_state, limit, and offset.
- The smoke opens the visible item from the browse grid, verifies item detail and Watch navigation, simulates playback progress, then returns to the prior browse route with browser history.
- The original browse URL state must still include the combined filter/sort/pagination params after returning.
- The browse data source must be called again with the original normalized browse query after returning.
- Playback progress must remain written to the selected source and Continue Watching/Home behavior must remain covered by existing smoke tests.
- Preserve the existing redaction guard: no raw ticket token, stream/source path, bearer token, or fingerprint appears in DOM text.

## Acceptance Criteria

- [x] Focused Media Web route test covers browse state -> item detail -> Watch -> progress write -> browser back to item detail -> browser back to browse state.
- [x] The test asserts the restored browse URL contains facet, sort, order, watch_state, limit, and offset.
- [x] The test asserts `listItems` receives the restored rich browse query.
- [x] The test asserts `updateUserPlaybackProgress` receives the selected source and position.
- [x] The test asserts unsafe playback internals are not rendered.
- [x] `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx` passes.
- [x] `npm run check --prefix apps/admin-web` passes.

## Definition of Done

- Route smoke added or adjusted in `mediaSurface.test.tsx`.
- No backend/API/SDK contract changes unless the route smoke proves a blocker.
- Docs/spec updated only if this task establishes a new durable route behavior beyond existing specs.
- Trellis context files configured before implementation.

## Technical Approach

Add a narrow Vitest/React Testing Library route smoke near the existing Media Web browse/playback tests. Use fixture mode and spy wrappers around `listItems` and `updateUserPlaybackProgress`. Start from a rich `/media/items?...` URL that returns `Pilot`, click the item card link, click `Watch`, simulate a `timeUpdate`, then invoke browser history back twice and assert the browse route/search plus data-source query are restored.

## Decision (ADR-lite)

**Context**: Existing tests validate browse state and playback separately, plus a library browse-to-playback happy path. The remaining risk is cross-route state restoration when users enter playback from a filtered browse result.

**Decision**: Use browser history restoration as the MVP instead of adding `return_to` plumbing or redesigning item/watch navigation.

**Consequences**: The smoke validates current route-owned behavior with minimal product surface change. A future explicit Back to results affordance can be designed later if product UX needs state restoration outside browser history.

## Out of Scope

- New backend browse/search APIs.
- New route params such as `return_to`.
- Player redesign, breadcrumbs, or broader navigation UX changes.
- Live-server E2E or browser/device compatibility matrix.
- Native/TV/casting clients.

## Technical Notes

- Relevant spec: `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`.
- Relevant files inspected:
  - `apps/admin-web/src/App.tsx`
  - `apps/admin-web/src/surfaces/media/MediaPages.tsx`
  - `apps/admin-web/src/surfaces/media/MediaItemShared.tsx`
  - `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
- Existing helper candidates:
  - `createBrowseListItemsMock`
  - `setMediaTiming`
  - `findMediaPanelSection`
