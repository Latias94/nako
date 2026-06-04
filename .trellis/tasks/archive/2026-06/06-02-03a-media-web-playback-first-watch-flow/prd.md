# Media Web Playback First Watch Flow

## Goal

Make the browser Media Web surface complete one practical first-watch flow from
catalog detail to playback start using Nako's public client and playback
contracts.

## Requirements

- Audit the current `web/src/features/media` playback path, public data source,
  and `video-player` behavior before changing code.
- Use Public Client API and SDK/client-core request builders where available;
  do not call Admin routes for user playback.
- Support the first viable playback start path for Direct Play, Remux, or HLS
  based on existing playback decision/session contracts.
- Keep bearer tokens and playback tickets redacted in visible UI state, route
  state, logs, and tests.
- Preserve Media Web layout quality on desktop and mobile.
- Add route/data-source/component tests for the chosen watch flow.

## Acceptance Criteria

- [x] A user can navigate from Media Web item/detail context into a playback
  route or player state.
- [x] Playback decision/session request construction uses public playback
  contracts and safe previews.
- [x] The player handles at least one realistic playable URL mode without
  exposing tokens.
- [x] Tests cover route state, data-source mapping, and player action behavior.
- [x] Build/bundle checks for the touched Web package pass.
- [x] Browser or Playwright smoke evidence is recorded if UI behavior changes.

## Definition of Done

- Focused Web tests pass.
- Relevant Rust public-client/playback contract tests pass if touched.
- Redaction and route-state behavior are explicitly tested.
- PRD/evidence notes are updated with commands run and remaining follow-ons.

## Evidence

- Worker evidence:
  - `npm run check --prefix web`
  - `npm run test --prefix web -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx src/test/video-player.test.tsx src/test/data-source-contracts.test.ts`
  - `npm run build:budget --prefix web`
  - `cargo nextest run -p nako-client-protocol -p nako-client-core -p nako-client --no-fail-fast`
  - `npx --no-install playwright-cli goto "http://127.0.0.1:3000/media/watch?id=live-movie&type=movie&source_id=source-live"`
  - `npx --no-install playwright-cli --raw eval "document.body.textContent?.includes('视频播放区域')"` returned `true`.
  - `npx --no-install playwright-cli --raw eval "location.href"` returned `http://127.0.0.1:3000/media/watch?id=live-movie&type=movie&source_id=source-live`.
- Fresh integration evidence on 2026-06-02 after syncing `main`:
  - `npm run check --prefix web` passed.
  - `npm run test --prefix web -- src/test/route-contracts.test.tsx src/test/route-state-contracts.test.tsx src/test/video-player.test.tsx src/test/data-source-contracts.test.ts` passed: 4 files, 112 tests. jsdom printed a non-fatal `HTMLMediaElement.pause` not implemented warning.
  - `npm run build:budget --prefix web` passed. `total-js` gzip was `344.89 KiB` against the `345 KiB` limit.
  - `cargo nextest run -p nako-client-protocol -p nako-client-core -p nako-client --no-fail-fast` passed: 43 tests.
  - `npx --no-install playwright-cli open "http://127.0.0.1:3000/media/watch?id=live-movie&type=movie&source_id=source-live"` loaded the 03a worktree route from the local Vite server.
  - `npx --no-install playwright-cli eval '() => location.href'` returned `http://127.0.0.1:3000/media/watch?id=live-movie&type=movie&source_id=source-live`.
  - `npx --no-install playwright-cli eval '() => /public-token|video-ticket/.test(document.body.innerText + location.href)'` returned `false`.
  - The browser smoke used fixture fallback data, so native `<source>` injection is verified by the focused route/data-source/player tests above rather than by the unauthenticated local Vite page.

## Follow-ons

- HLS.js/Shaka-backed browser playback remains a separate player UX follow-on;
  this task wires the first native playable ticket URL path.
- Numeric fixture IDs still normalize through TanStack Router defaults; live
  public item/source IDs remain string-safe and are covered by route tests.

## Out of Scope

- No new playback engine implementation.
- No TV/casting feature expansion.
- No Admin-only player workflow.
- No schema migration unless a blocker proves it is required and planner
  approves the shared scope.

## Technical Notes

- Likely Web files: `web/src/features/media/video-player.tsx`,
  `web/src/features/media/media-detail.tsx`,
  `web/src/api/public/media-data-source.ts`, and route tests under
  `web/src/test`.
- Coordinate with the playback runtime lane before changing server playback
  session semantics.
- If public contract changes are required, stop and return to planner
  coordination before widening scope.
