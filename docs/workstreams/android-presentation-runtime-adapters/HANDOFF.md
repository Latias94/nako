# Android Presentation Runtime Adapters - Handoff

Status: Closed
Last updated: 2026-05-20

## Current Task

Closed.

## Notes

- Keep responses to the user in Chinese; keep code and technical docs in
  English.
- Do not touch generated `output/` or `tmp/` directories.
- APRA-010 through APRA-040 are complete.
- `ArtworkRequestResolver` now hides token-backed artwork request creation from
  Home, Libraries, and Media Item Detail visual APIs.
- `PlayerRouteRenderer` now hides concrete player route dependencies from
  `TaruBrowseShell`.
- Player lifecycle ownership remains explicitly reserved for
  `android-player-session-architecture`.
- Final gates passed on 2026-05-20:
  - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
  - `git diff --check`

## Follow-ons

- `docs/workstreams/android-player-session-architecture/` owns ExoPlayer,
  player state, retry/error reducer, and exit side effects.
