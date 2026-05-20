# Android Player Route Host - TODO

Status: Closed
Last updated: 2026-05-20

## Task Ledger

- [x] APRH-010 - Freeze host boundary.
  - Owner: Codex
  - Scope: Workstream docs only.
  - Validation: Docs describe target host and non-goals.

- [x] APRH-020 - Add tested player route host.
  - Owner: Codex
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/PlayerRouteHost.kt`
    - `apps/android/app/src/test/java/dev/taru/android/ui/screens/player/PlayerRouteHostTest.kt`
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.PlayerRouteHostTest --no-daemon --no-parallel`
  - Evidence: Passed on 2026-05-20.

- [x] APRH-030 - Move route orchestration out of `PlaybackPlayerRoute`.
  - Owner: Codex
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/screens/player/PlaybackPlayerRoute.kt`
  - Validation:
    - Focused player tests pass.
  - Evidence: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.* --no-daemon --no-parallel` passed on 2026-05-20.

- [x] APRH-040 - Verify and close lane.
  - Owner: Codex
  - Scope: Workstream docs and validation.
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.* --no-daemon --no-parallel`
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
    - `git diff --check`
  - Evidence: Final gate passed on 2026-05-20.
