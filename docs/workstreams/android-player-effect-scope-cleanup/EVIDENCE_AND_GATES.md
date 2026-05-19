# Android Player Effect Scope Cleanup - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Required Gates

- Focused Android JVM tests for scope wiring.
- Existing player exit and playback tests continue passing.
- Full `:app:testDebugUnitTest`.
- `git diff --check`.

## Evidence Log

- 2026-05-19: Lane opened. Implementation gates pending.
- 2026-05-19: Added `launchPlayerExitEffect` and
  `PlayerExitEffectScopeTest`; verified exit work uses the injected scope and
  respects scope cancellation.
- 2026-05-19: Routed `playerExitEffectScope` from `TaruAndroidApp` through
  `TaruBrowseShell` to `PlaybackPlayerRoute`; removed route-local
  `CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)`.
- 2026-05-19: Ran
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.PlayerExitEffectScopeTest --tests dev.taru.android.player.PlaybackExitCoordinatorTest --no-daemon --no-parallel`.
  Result: pass.
- 2026-05-19: Ran
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.PlayerExitEffectScopeTest --tests dev.taru.android.ui.screens.player.PlayerPresentationTest --tests dev.taru.android.player.PlaybackExitCoordinatorTest --tests dev.taru.android.player.PlaybackExitEffectsTest --no-daemon --no-parallel`.
  Result: pass.
- 2026-05-19: Ran
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`.
  Result: pass.
- 2026-05-19: Ran `git diff --check`. Result: pass.

## Residual Follow-Ons

- No immediate follow-on remains. A future app architecture pass may introduce
  a typed `AndroidAppEnvironment`, but this lane intentionally avoided that
  broader container refactor.
