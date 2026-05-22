# Android Root App Composition - Handoff

Status: Closed
Last updated: 2026-05-20

## Current State

ARAC-010 through ARAC-040 are complete. The lane is closed.

## Active Task

No active task remains in this workstream.

## File Scope

- `apps/android/app/src/main/java/dev/nako/android/ui/`
- `apps/android/app/src/test/java/dev/nako/android/ui/`
- Workstream docs under this directory.

## Validation

Run:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.NakoAppCompositionTest --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
git diff --check
```

## Notes

- Keep this a plain Kotlin composition module.
- Do not add a DI framework.
- Do not touch generated `output/` or `tmp/`.
- Future root-wide concerns such as auth/session refresh, app-wide preference
  composition, dynamic color policy, or telemetry should enter through
  `NakoAppEnvironment` or a successor root composition module.
