# Media Web Source Continuity Playback Smoke

## Goal

Prove that a user can switch playback source versions in Media Web and keep the selected source consistent across the Watch URL, browser playback ticket, progress writes, and Continue Watching resume links.

## What I Already Know

- Existing item detail tests cover selecting `source-episode-1-alt`, updating the URL, previewing playback decisions, and writing watched state with the selected source.
- Existing watch tests cover direct-entry playback progress writes for `source-episode-1-alt`, ticket retry behavior, source-specific candidate reset, and auto-resume mismatch messaging.
- Existing Continue Watching tests cover resume links preserving the saved source.
- The missing user-level proof is the combined Watch-page source switch flow: start on one source, switch source in the player page, then verify ticket/progress/Continue Watching stay on the switched source.
- Admin Web Media Web tests must not render raw stream URLs, ticket tokens, source paths, bearer tokens, or fingerprints.

## Assumptions

- This should be a route-level Admin Web smoke, not a backend/API/SDK change.
- The MVP can use fixture mode and spy wrappers around existing Media Web data-source methods.
- The route already has the right product controls; this task should only add a small test unless inspection proves a real gap.

## Open Questions

- None.

## Requirements

- Add a Media Web route smoke that starts at `/media/watch/item-episode-1?source_id=source-episode-1`.
- The smoke switches to `Pilot.alt.mp4` from the Watch page source versions panel.
- The Watch URL must update to `source_id=source-episode-1-alt`.
- Browser playback ticket creation must be requested for `source-episode-1-alt` after switching.
- Playback progress must be written with `source_id: "source-episode-1-alt"` after switching and starting playback.
- Returning Home must show Continue Watching with a Resume link for `source-episode-1-alt`.
- Preserve redaction guards for ticket/source internals in rendered DOM text.

## Acceptance Criteria

- [x] A focused route test covers Watch source switch -> alternate ticket -> progress write -> Home Continue Watching resume link.
- [x] The test asserts the URL changes to `source_id=source-episode-1-alt`.
- [x] The test asserts `createBrowserPlaybackTicket` is called for `source-episode-1-alt`.
- [x] The test asserts `updateUserPlaybackProgress` is called with `source_id: "source-episode-1-alt"`.
- [x] The test asserts Home Resume links to `/media/watch/item-episode-1?source_id=source-episode-1-alt`.
- [x] The test asserts unsafe playback internals are not rendered.
- [x] `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx` passes.
- [x] `npm run check --prefix apps/admin-web` passes.

## Definition of Done

- Route smoke added in `mediaSurface.test.tsx` or an adjacent focused test file if repo patterns support it.
- No production behavior changes unless the smoke exposes a real source continuity gap.
- Docs/spec updated only if a new durable route/player contract is established.
- Trellis context files configured before implementation.

## Technical Approach

Add a narrow Vitest/React Testing Library smoke near the existing Watch/player source tests. Use fixture mode, spy on `createBrowserPlaybackTicket` and `updateUserPlaybackProgress`, switch source via the existing `Pilot.alt.mp4` button, simulate playback progress, then navigate Home and assert Continue Watching source continuity.

## Decision (ADR-lite)

**Context**: Source-specific behavior already has unit-like route tests, but not a complete user path that starts from a Watch source switch and ends at Continue Watching.

**Decision**: Validate the current route-owned `source_id` and playback state model with a route smoke instead of adding new return/resume plumbing.

**Consequences**: This increases confidence in multi-version playback without changing product UI. Future work can still add richer source labels or source-aware resume controls if needed.

## Out of Scope

- New source-selection UI or player redesign.
- Backend playback planning changes.
- Public Client SDK changes.
- New route params beyond existing `source_id`.
- Live-server E2E or device compatibility coverage.

## Technical Notes

- Relevant spec: `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`.
- Relevant files inspected:
  - `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx`
  - `apps/admin-web/src/surfaces/media/MediaItemShared.tsx`
  - `apps/admin-web/src/surfaces/media/MediaItemDetailPage.tsx`
  - `apps/admin-web/src/surfaces/media/MediaWatchPage.tsx`
- Existing helper candidates:
  - `setMediaTiming`
  - `createFixtureMediaDataSource`
