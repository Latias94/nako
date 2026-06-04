# Media Web playback first-watch integration hardening

## Goal

Harden the existing Media Web first-watch path so browser playback requests use
client capability facts consistently and the watch page exposes safer playback
state when the ticketed URL cannot be played.

## What I Already Know

- `03a-media-web-playback-first-watch-flow` already shipped the first route,
  ticketed `<video>` playback path, source selection, progress writes, watched
  updates, and token redaction tests.
- `docs/architecture/PLAYBACK.md` lists Web player integration follow-ons:
  capability reporting, richer retry UX, and desktop/native player decisions.
- `apps/admin-web` currently has no HLS library dependency. The player uses a
  native `<video src=...>` path.
- `@nako/sdk` already exposes `PlaybackCapabilitiesQuery` and
  `BrowserPlaybackTicketRequest.capabilities`, so this first slice should not
  need a Public Client protocol change.

## Requirements

- Derive a browser playback capability profile in the Media Web surface instead
  of hard-coding only `{ direct_play: true }` and broad ticket capabilities.
- Reuse the same capability profile for playback decision and browser ticket
  creation where the wire shapes overlap.
- Keep ticket URLs and bearer tokens out of visible text, route state, and test
  failure output.
- Add user-visible player failure/retry state for the ticketed video element.
- Preserve existing source selection, progress write, pause flush, and ended
  watched behavior.

## Acceptance Criteria

- [ ] Media Web tests prove capability facts are sent to playback decision and
  browser ticket requests.
- [ ] Media Web tests prove playback URL/ticket secrets remain hidden from
  rendered text.
- [ ] Media Web tests prove player error state can be retried without changing
  route state or leaking the URL.
- [ ] Existing playback progress and watched-state tests still pass.
- [ ] `npm run check --prefix apps/admin-web` passes.
- [ ] Focused Media Web tests pass.

## Definition of Done

- No new dependency unless a blocker proves native browser handling is
  insufficient for this slice.
- No Public Client DTO/schema changes unless planning is reopened.
- No server playback runtime behavior changes.
- Validation commands and remaining follow-ons are recorded before completion.

## Out of Scope

- No hls.js/Shaka integration in this slice.
- No desktop/native external player selection.
- No device-profile database or persisted per-user playback preferences.
- No playback session schema migration.

## Technical Approach

- Keep changes inside `apps/admin-web/src/surfaces/media`.
- Add a small browser capability helper that is testable under jsdom.
- Feed the helper into `getPlaybackDecision` and
  `createBrowserPlaybackTicket`.
- Add local video element error state and a retry command that remounts/reloads
  the video element without exposing the ticket URL as text.

## Technical Notes

- Existing Media surface files:
  `MediaPages.tsx`, `mediaDataSource.ts`, `mediaSurface.test.tsx`, and
  `mediaDataSource.test.ts`.
- Existing frontend validation:
  `npm run check --prefix apps/admin-web` and focused Vitest media tests.

## Verification

- Passed: `npm run check --prefix apps/admin-web`.
- Passed:
  `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx src/surfaces/media/mediaDataSource.test.ts`.
- Passed: `npm run build --prefix apps/admin-web`.
- Passed: `git diff --check`.
- Playwright smoke passed against `http://127.0.0.1:5177/media/watch/item-episode-1?source_id=source-episode-1-alt`:
  after choosing fixture mode, the watch page rendered the `Pilot` player, the
  ticket/stream path stayed out of page text, and the safe retry state appeared
  when the fixture URL failed to load.
