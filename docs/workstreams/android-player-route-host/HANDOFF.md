# Android Player Route Host - Handoff

Status: Closed
Last updated: 2026-05-20

## Current State

APRH-010 through APRH-040 are complete. The lane is closed.

## Active Task

None.

## File Scope

- `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/`
- `apps/android/app/src/test/java/dev/taru/android/ui/screens/player/`
- Workstream docs under this directory.

## Validation

Run:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.PlayerRouteHostTest --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
git diff --check
```

## Notes

- Keep `PlayerSession` as reducer/state machine.
- `PlayerRouteHost` now owns route lifecycle orchestration and listener event
  mapping.
- `PlaybackPlayerRoute` keeps Android context access, host construction,
  `PlayerView` binding, overlay rendering, and clipboard UI actions.
- Do not touch generated `output/` or `tmp/`.

## Residual Risks

- No follow-on is required for this lane. Track/subtitle selection remains a
  separate future player UX scope.
