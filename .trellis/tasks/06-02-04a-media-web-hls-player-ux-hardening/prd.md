# Media Web HLS Player UX Hardening

## Goal

Make the Media Web player more practical for browser playback by supporting a
bounded HLS playback path and improving player-state behavior without exposing
bearer tokens or playback tickets.

## Requirements

- Audit the current Media Web watch route, `VideoPlayer`, public data source,
  playback tickets, subtitles, and heartbeat behavior before changing code.
- Support the first viable HLS playback path for browsers that cannot play the
  playlist natively. Prefer a proven browser HLS engine and lazy-load it so the
  existing bundle budget remains credible.
- Keep Direct Play/native source playback working through the existing video
  element path.
- Preserve redaction: bearer tokens and playback tickets must not appear in
  visible UI, route state, logs, or safe previews.
- Keep subtitles, playback session heartbeat/cancel behavior, and route-state
  tests coherent with the Public Client API.
- Preserve desktop and mobile layout quality.

## Acceptance Criteria

- [ ] HLS playlist playback is handled by native browser support or a lazy
  loaded HLS engine with a tested fallback/error state.
- [ ] Direct playable URLs still render through the native video element.
- [ ] Route state and DOM do not expose bearer tokens or playback ticket values.
- [ ] Tests cover the HLS path, native path, subtitles, and player lifecycle.
- [ ] Bundle budget and Web package checks pass.
- [ ] Browser or Playwright smoke evidence is recorded.

## Definition of Done

- Focused Web tests pass.
- `npm run check --prefix web` and `npm run build:budget --prefix web` pass.
- Relevant Rust Public Client or playback contract tests pass if touched.
- Evidence notes record commands, smoke result, and follow-ons.

## Out of Scope

- No DRM.
- No TV/casting expansion.
- No broad player redesign.
- No Admin-only playback workflow.
- No server playback contract change without planner coordination.

## Technical Notes

- Likely files: `web/src/features/media/video-player.tsx`,
  `web/src/features/media/media-surface.tsx`, `web/src/api/public/media-data-source.ts`,
  and Web route/player tests.
- The current `total-js` gzip budget was close to the limit after 03a; avoid
  eagerly importing heavy player dependencies.
- Coordinate with playback runtime before changing server session semantics.
