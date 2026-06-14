# Media Web Playback State Loop

## Goal

Close the visible Media Web playback state loop for the current Admin Web validation surface: a user should be able to see active User Playback State on the Media home page, resume the same Media Item and Media Source through the watch route, and see clear resume/progress context on the watch page while existing progress write, pause flush, and watched-state behavior remains intact.

## Requirements

- Add a Continue Watching resume affordance on `/media` that routes to `/media/watch/$itemId`.
- Preserve the entry's `state.source_id` in the watch route search when it is available.
- Show redaction-safe resume context on Continue Watching rows: title, percent complete, resume position, and source continuity.
- Make `/media/watch/$itemId` visibly communicate the current resume position/progress before playback starts.
- Keep all playback transport secrets, ticket tokens, stream URLs, fingerprints, and raw source internals out of rendered page text.
- Reuse the existing `MediaWebDataSource`, `useMediaItemPlayback`, `MediaPlaybackState`, and fixture/live boundaries.

## Acceptance Criteria

- [ ] Continue Watching rows include a Resume action with an href like `/media/watch/item-episode-1?source_id=source-episode-1`.
- [ ] The Resume action keeps route/search state URL-owned and does not call direct `fetch`.
- [ ] The Watch page shows redaction-safe resume progress from User Playback State.
- [ ] Existing Watch page playback progress writes remain gated on actual playback start.
- [ ] Pause flush and ended watched-state tests still pass.
- [ ] Admin Web typecheck and focused Media Web tests pass.

## Definition of Done

- Tests added or updated for the resume route affordance and Watch page resume context.
- `npm run check --prefix apps/admin-web` passes.
- Focused Media Web Vitest coverage passes.
- Trellis task context validates.
- No unrelated user changes are reverted.

## Technical Approach

Implement this as a frontend-only Admin Web slice. Existing Public Client API coverage already includes `listContinueWatching`, `getUserPlaybackState`, `updateUserPlaybackProgress`, and `setUserWatchedState`; Watch page progress write/pause/ended behavior already exists. The highest-value missing product loop is navigation and visible context.

The implementation should:

- Keep `/media` loading Continue Watching through `dataSource.listContinueWatching()`.
- Convert each Continue Watching row into a row with a Resume link/button, using TanStack Router `Link`.
- Pass `source_id` through the watch route search only when present.
- Add a compact resume banner/fact row on the Watch page using `playbackState.value?.state`.
- Keep source selection still owned by `MediaItemSearch.source_id` and existing `onSearchChange`.

## Decision (ADR-lite)

**Context**: The backend and SDK already expose User Playback State and Continue Watching. The Watch page already writes progress after playback starts, flushes on pause, and marks watched on ended. Starting with automatic video seek would add browser metadata timing complexity without proving the basic product loop.

**Decision**: Ship a Resume navigation polish MVP: Continue Watching links to the Watch page with source continuity, and Watch page displays resume/progress context. Do not auto-seek video in this task.

**Consequences**: This creates a visible, low-risk playback state loop now. A later task can add auto-resume seek after defining browser metadata timing, user override behavior, and fixture/live test expectations.

## Out of Scope

- Automatic video seek to `resume_position_ms`.
- Backend API, SDK, database, or playback runtime changes.
- Persistent cross-route fixture mutation state.
- Multi-device conflict resolution, heartbeat buffering, or sync semantics.
- Public release frontend redesign beyond the current Admin Web validation surface.

## Technical Notes

- `apps/admin-web/src/surfaces/media/MediaPages.tsx` already loads Continue Watching.
- `apps/admin-web/src/surfaces/media/MediaWatchPage.tsx` already handles browser ticket retry, progress writes, pause flush, and ended watched-state writes.
- `apps/admin-web/src/surfaces/media/MediaItemShared.tsx` owns shared item playback state UI and source selection.
- `apps/admin-web/src/surfaces/media/mediaDataSource.ts` already maps live and fixture Public Client playback state APIs.
- `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx` already covers progress throttling, pause flush, watched updates, and ticket redaction.
- Relevant specs:
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
