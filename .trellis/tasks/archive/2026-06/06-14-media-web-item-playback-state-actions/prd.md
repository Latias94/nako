# Media Web Item Playback State Actions

## Goal

Make Media Item detail a useful playback-state action surface, not just a read-only status panel. Users should be able to resume the saved source or clear progress from the item page, while Continue Watching remains the result view that reflects those mutations.

## Requirements

- Media Item detail must expose a Resume action when User Playback State has an active resume position.
- Resume must preserve the saved `source_id` from User Playback State, even when the currently selected source differs.
- Media Item detail must expose a Start over action when the item has active progress or watched state.
- Start over must clear the resume position by writing User Playback State through the existing Media Web data source.
- Existing Mark watched and Mark unwatched behavior must continue to use the Public Client playback-state mutation boundary.
- Continue Watching must reflect item-detail mutations in fixture mode.
- Keep backend, SDK, database, and Public Client API unchanged.
- Preserve redaction rules: do not render browser-ticket tokens, stream URLs, bearer tokens, fingerprints, raw paths, or source internals.

## Acceptance Criteria

- [x] Item detail renders Resume when fixture playback state has an active resume position.
- [x] Resume links to `/media/watch/$itemId` with the playback-state `source_id`, not necessarily the selected source.
- [x] Start over clears fixture resume state from item detail and removes the item from active Continue Watching.
- [x] Mark watched / unwatched still call `setUserWatchedState` with the selected source and safe payloads.
- [x] Existing Media Web watch, auto-resume, Continue Watching refresh, and redaction tests still pass.
- [x] Admin Web check, focused Media Web tests, full Admin Web tests, and build pass.

## Definition of Done

- Tests added or updated for Resume and Start over item-detail behavior.
- `npm run check --prefix apps/admin-web` passes.
- Focused Media Web Vitest tests pass.
- `npm run test --prefix apps/admin-web` passes.
- `npm run build --prefix apps/admin-web` passes.
- Trellis task validates, is archived after completion, and the journal records the session outcome.

## Technical Approach

Reuse the existing Media Web page boundary:

- Keep route/search state URL-owned.
- Extend `useMediaItemPlayback()` with a `startOver()` mutation that calls `setUserWatchedState(itemId, { watched: false, position_ms: 0, duration_ms, source_id })`.
- Extend `MediaPlaybackState` with a Start over button using the existing mutation state/error plumbing.
- Add an item-detail Resume link that derives its `source_id` from `playbackState.value.state.source_id` when there is an active resume position.

No new state manager, global cache, backend route, or SDK contract is required.

## Decision (ADR-lite)

**Context**: Watch page already owns browser playback, progress writes, and auto-resume. Item detail already loads playback state and can mutate watched state, but it does not expose the saved-source Resume path or a persistent Start over/reset action.

**Decision**: Keep the action surface in the existing item playback shared hook and shared playback-state component.

**Consequences**: The implementation stays narrow and consistent with current Media Web route splitting. A later richer client may introduce a broader playback-state action model, but this slice proves the Public Client API behavior without adding a frontend state framework.

## Out of Scope

- Backend, SDK, database, or Public Client API changes.
- Cross-tab synchronization or live cache invalidation.
- A global Media Web state store.
- New Home page controls beyond verifying Continue Watching reflects mutations.
- Browser-player Start over behavior, which already exists on the Watch page.

## Technical Notes

- Specs read:
  - `.trellis/spec/admin-web/frontend/index.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/index.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
- Relevant code inspected:
  - `MediaItemDetailPage.tsx`
  - `MediaItemShared.tsx`
  - `MediaWatchPage.tsx`
  - `MediaPages.tsx`
  - `mediaDataSource.ts`
  - `mediaSurface.test.tsx`
