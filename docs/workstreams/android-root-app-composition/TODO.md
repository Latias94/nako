# Android Root App Composition - TODO

Status: Closed
Last updated: 2026-05-20

## Task Ledger

- [x] ARAC-010 - Freeze root composition boundary.
  - Owner: Codex
  - Scope: Workstream docs only.
  - Validation: Docs describe target root composition and non-goals.
  - Handoff: Boundary recorded in `DESIGN.md`.

- [x] ARAC-020 - Add root app environment and runtime module.
  - Owner: Codex
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/TaruAppComposition.kt`
    - `apps/android/app/src/test/java/dev/taru/android/ui/TaruAppCompositionTest.kt`
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.TaruAppCompositionTest --no-daemon --no-parallel`
  - Handoff: Root environment owns client/store graph and can create `TaruAppSession`.

- [x] ARAC-030 - Simplify `TaruAndroidApp` root rendering.
  - Owner: Codex
  - Scope:
    - `apps/android/app/src/main/java/dev/taru/android/ui/TaruAndroidApp.kt`
  - Validation:
    - Focused root app/session tests pass.
  - Handoff: `TaruAndroidAppContent` takes root environment/session instead of individual clients.

- [x] ARAC-040 - Verify and close lane.
  - Owner: Codex
  - Scope: Workstream docs and validation.
  - Validation:
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.* --no-daemon --no-parallel`
    - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
    - `git diff --check`
  - Handoff: Close or split follow-ons.
