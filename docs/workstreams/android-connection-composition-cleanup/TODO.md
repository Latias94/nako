# Android Connection Composition Cleanup - TODO

Status: Closed
Last updated: 2026-05-20

## Task Ledger

- [x] ACCC-010 - Freeze cleanup boundary.
  - Owner: Codex
  - Scope: Workstream docs only.
  - Validation: Docs describe target cleanup and non-goals.

- [x] ACCC-020 - Move connection runtime creation to root environment.
  - Owner: Codex
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/TaruAppComposition.kt`
    - `apps/android/app/src/test/java/dev/taru/android/ui/TaruAppCompositionTest.kt`
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.TaruAppCompositionTest --no-daemon --no-parallel`

- [x] ACCC-030 - Remove duplicate connection shell platform entrypoint.
  - Owner: Codex
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/TaruAndroidApp.kt`
    - `apps/android/app/src/main/java/dev/taru/android/ui/connection/TaruConnectionShell.kt`
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.connection.* --tests dev.taru.android.ui.* --no-daemon --no-parallel`

- [x] ACCC-040 - Verify and close lane.
  - Owner: Codex
  - Scope: Workstream docs and validation.
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
    - `git diff --check`
