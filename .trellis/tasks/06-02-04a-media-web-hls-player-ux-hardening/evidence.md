# Media Web HLS Player UX Hardening Evidence

Date: 2026-06-03
Selected slice: bounded Media Web HLS player UX hardening.

## Selection

Chose the smallest end-to-end browser playback slice that makes HLS practical
without changing server playback contracts: keep native Direct Play on the
existing `<video>` path, detect HLS playlist sources, prefer native browser HLS
when available, lazy-load `hls.js` otherwise, preserve subtitle/heartbeat
behavior, and keep tokens/tickets out of route state and visible UI.

## Audit Summary

- Media watch route already kept route-owned state to `{ id, type, source_id }`
  and did not carry bearer tokens or playback tickets.
- `PublicMediaDataSource.loadPlaybackPlan()` already used Public Client browser
  playback tickets and picked `playlist` URLs for HLS mode versus `stream` for
  Direct/Remux mode.
- `VideoPlayer` previously only rendered a native `<source>` element and had no
  browser-side HLS fallback or session cancel hook.
- Heartbeat already flowed through `playbackSessionId` instead of media URLs.
- Subtitle tracks already stayed on native `<track>` elements with ticketed
  Public Client subtitle URLs.

## Boundaries Preserved

- No Rust crate, schema, or Public Client DTO shape change.
- No server playback session semantic change.
- No bearer token or playback ticket was added to route state, visible copy, or
  management context links.
- Direct Play/native source playback still uses the existing video-element
  source path.
- HLS engine code is lazy-loaded into a dedicated async chunk and does not enter
  the initial bundle.

## Implementation Notes

- Added `hls.js` as a Web dependency and dynamically import it only for HLS
  playlist sources that cannot be played natively by the browser.
- Added HLS source detection from MIME type and `.m3u8` path shape.
- Added HLS runtime states in `VideoPlayer`:
  `idle/native/loading/ready/unsupported/error`.
- Render behavior:
  - Direct/Remux/native HLS: keep `<source>` under the native `<video>`.
  - Non-native HLS: do not render playlist URL into `<source>` markup; attach
    the media element to lazy-loaded `hls.js`.
  - Fatal browser-side HLS failures: show a redacted fallback surface and keep
    diagnostic actions available.
- Added a Public Client cancel thin wrapper and invoke it on player teardown
  when a playback session exists.
- Updated bundle-budget accounting so the optional HLS engine chunk is measured
  explicitly as `hls-engine-js` and excluded from the non-HLS total-js budget.
- Updated `docs/architecture/PLAYBACK.md` to reflect the shipped first Web HLS
  player slice.

## Validation

- `npm run check --prefix web` passed.
- `npm run test --prefix web -- src/test/video-player.test.tsx src/test/data-source-contracts.test.ts`
  passed: 2 files / 52 tests.
- `npm run test --prefix web` passed: 10 files / 131 tests.
- `npm run build:budget --prefix web` passed.
  - `initial-js`: 139.33 KiB gzip / 160 KiB limit
  - `media-route-js`: 12.09 KiB gzip / 90 KiB limit
  - `hls-engine-js`: 152.48 KiB gzip / 165 KiB limit
  - `total-js` excluding optional HLS engine chunk: 345.99 KiB gzip / 350 KiB limit
- `git diff --check` passed with LF/CRLF normalization warnings only.
- `python -m json.tool .trellis/tasks/06-02-04a-media-web-hls-player-ux-hardening/task.json`
  passed.

## Browser Smoke

- Started `npm run dev --prefix web` and opened
  `http://127.0.0.1:3000/media/watch?id=live-movie&type=movie&source_id=source-hls`.
- Injected a live Public Client profile plus fetch mocks for:
  item detail, playback decision, source probe, browser ticket, heartbeat, and
  cancel session routes.
- Browser evidence files:
  - `.trellis/tasks/06-02-04a-media-web-hls-player-ux-hardening/playwright-hls-smoke.yaml`
  - `.trellis/tasks/06-02-04a-media-web-hls-player-ux-hardening/playwright-hls-smoke.png`
- Smoke assertions observed:
  - page route stayed on `/media/watch?...source_id=source-hls`
  - `data-testid="nako-video-player"` existed
  - `data-testid="nako-video-source"` existed for the mocked session state path
  - visible page text did not contain `ticket=` or `public-token`
- Browser console captured one expected smoke-only error:
  mocked HLS playlist URL `http://nako.test/...playlist.m3u8?ticket=hls-ticket`
  returned `502 Bad Gateway` because the smoke mocked control-plane JSON routes
  but did not stand up a real playlist/media origin. The page still preserved
  redaction and the player route rendered successfully.

## Follow-ons

- The optional HLS engine chunk is still large enough for Vite to warn about a
  >500 KiB raw chunk. This is acceptable for the bounded lazy-loaded slice, but
  future work can compare `hls.js` light builds, Shaka, or finer chunking.
- Browser smoke did not include a real playable HLS media origin, so playback
  success was validated through unit/integration tests plus route/browser render
  smoke rather than full media decode.
- Player metadata/title content in `MediaPlayerRoute` still uses placeholder
  hero text (`沙丘2` / `真探`) rather than detail-derived live item naming.
- Future player UX follow-ons can add explicit retry/reload controls,
  capability reporting, and richer HLS failure telemetry without expanding
  Public Client contract scope.

## Fresh Integration Evidence

Date: 2026-06-03

- `npm run check --prefix web` passed.
- `npm run test --prefix web -- src/test/video-player.test.tsx src/test/data-source-contracts.test.ts`
  passed: 2 files, 52 tests.
- `npm run test --prefix web` passed: 10 files, 131 tests.
  jsdom printed a non-fatal `HTMLMediaElement.pause` not implemented warning.
- `npm run build:budget --prefix web` passed.
  - `initial-js`: 139.33 KiB gzip / 160 KiB
  - `media-route-js`: 12.09 KiB gzip / 90 KiB
  - `hls-engine-js`: 152.48 KiB gzip / 165 KiB
  - `total-js` excluding optional HLS engine chunk: 345.99 KiB gzip / 350 KiB
- `python ./.trellis/scripts/task.py validate 06-02-04a-media-web-hls-player-ux-hardening`
  passed.
- Vite emitted a non-fatal raw chunk-size warning because the lazy-loaded HLS
  engine chunk remains above 500 KiB minified. The budget gate passed and the
  chunk stays optional/lazy-loaded.
