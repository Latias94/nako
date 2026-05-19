# Android Player Exit Effects Coordinator - Handoff

Status: Closed
Last updated: 2026-05-19

## Completed

- APEC-020: extracted player exit side-effect client wiring into
  `PlaybackExitCoordinator` and wired `PlaybackPlayerRoute` to it.
- APEC-030: verified targeted and full debug unit gates and closed the lane.

## Notes

- Preserve once-only exit behavior in `PlaybackPlayerRoute`.
- Preserve detached exit execution semantics for now; app-level coroutine scope
  can be considered separately.
- Do not touch generated `output/` or `tmp/` directories.

## Evidence

- `PlaybackExitCoordinatorTest`: unfinished session cancellation/progress,
  ended watched report, and missing-token local-only behavior.
- Adjacent tests passed: `PlaybackExitEffectsTest`, `TaruPlaybackClientTest`,
  and `TaruUserPlaybackClientTest`.
- Full Android debug unit test suite passed with
  `:app:testDebugUnitTest --no-daemon --no-parallel`.
- `git diff --check` passed.

## Next

No continuation task remains in this workstream.
