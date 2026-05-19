# Android Player Effect Scope Cleanup - Handoff

Status: Closed
Last updated: 2026-05-19

## Completed

- APESC-020: moved player exit coroutine ownership to the app shell and
  injected it into `PlaybackPlayerRoute`.
- APESC-030: verified targeted and full debug unit gates and closed the lane.

## Notes

- Preserve detached exit execution semantics; only the owner changes.
- Do not touch generated `output/` or `tmp/` directories.

## Evidence

- `PlayerExitEffectScopeTest`: injected scope usage and cancellation behavior.
- Adjacent player exit tests passed: `PlaybackExitCoordinatorTest`,
  `PlaybackExitEffectsTest`, and `PlayerPresentationTest`.
- Full Android debug unit test suite passed with
  `:app:testDebugUnitTest --no-daemon --no-parallel`.
- `git diff --check` passed.

## Next

No continuation task remains in this workstream.
