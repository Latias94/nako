# Android Clipboard API Cleanup - TODO

Status: Closed
Last updated: 2026-05-20

## Task Ledger

- [x] ACAC-010 - Confirm local clipboard API and freeze scope.
  - Owner: Codex
  - Scope: Workstream docs and local artifact inspection.
  - Validation: API evidence recorded in `DESIGN.md`.

- [x] ACAC-020 - Add internal clipboard adapter.
  - Owner: Codex
  - Scope:
    - `apps/android/app/src/main/java/dev/nako/android/ui/NakoClipboard.kt`
  - Validation:
    - Compile focused UI tests.

- [x] ACAC-030 - Replace settings/player deprecated clipboard usage.
  - Owner: Codex
  - Scope:
    - `apps/android/app/src/main/java/dev/nako/android/ui/screens/settings/SettingsScreens.kt`
    - `apps/android/app/src/main/java/dev/nako/android/ui/screens/player/PlaybackPlayerRoute.kt`
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.settings.* --tests dev.nako.android.ui.screens.player.* --no-daemon --no-parallel`

- [x] ACAC-040 - Verify and close lane.
  - Owner: Codex
  - Scope: Workstream docs and validation.
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
    - `git diff --check`
