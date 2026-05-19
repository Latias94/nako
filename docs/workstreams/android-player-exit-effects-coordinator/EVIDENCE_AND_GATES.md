# Android Player Exit Effects Coordinator - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Required Gates

- Focused Android JVM tests for player exit coordinator behavior.
- Existing `PlaybackExitEffectsTest` continues passing.
- Full `:app:testDebugUnitTest` when practical.
- `git diff --check`.
- Emulator smoke only if player runtime behavior changes beyond coordinator
  wiring.

## Evidence Log

- 2026-05-19: Lane opened. Implementation gates pending.
- 2026-05-19: Added `PlaybackExitCoordinator` and
  `PlaybackExitCoordinatorTest`. Focused tests cover unfinished session
  cancellation plus progress report, ended playback watched report without
  cancellation, and missing-token local-only behavior.
- 2026-05-19: Ran
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackExitCoordinatorTest --no-daemon --no-parallel`.
  Result: pass.
- 2026-05-19: Ran
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackExitCoordinatorTest --tests dev.taru.android.player.PlaybackExitEffectsTest --tests dev.taru.android.playback.TaruPlaybackClientTest --tests dev.taru.android.userplayback.TaruUserPlaybackClientTest --no-daemon --no-parallel`.
  Result: pass.
- 2026-05-19: Ran
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`.
  Result: pass.
- 2026-05-19: Ran `git diff --check`. Result: pass.

## Residual Follow-Ons

- App-level coroutine ownership for detached exit effects remains a possible
  future refinement, but was intentionally not changed in this lane.
- Emulator smoke was not rerun because this lane preserved the existing
  `applyPlaybackExitEffects` contract and changed only UI-to-coordinator
  wiring covered by JVM tests.
