# Android Connection Composition Cleanup - Handoff

Status: Closed
Last updated: 2026-05-20

## Current State

ACCC-010 through ACCC-040 are complete. The lane is closed.

## Active Task

No active task remains in this workstream.

## File Scope

- `apps/android/app/src/main/java/dev/nako/android/ui/`
- `apps/android/app/src/main/java/dev/nako/android/ui/connection/`
- `apps/android/app/src/test/java/dev/nako/android/ui/`
- Workstream docs under this directory.

## Validation

Run:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.NakoAppCompositionTest --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.connection.* --tests dev.nako.android.ui.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
git diff --check
```

## Notes

- Root environment owns platform dependency composition.
- Connection content should accept runtime, not construct runtime dependencies.
- Do not touch generated `output/` or `tmp/`.
- Future connection behavior should enter through `ConnectionSession` and
  `ConnectionRuntime`, with production runtime assembly at `NakoAppEnvironment`.
