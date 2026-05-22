# Android Clipboard API Cleanup - Handoff

Status: Closed
Last updated: 2026-05-20

## Current State

ACAC-010 through ACAC-040 are complete. The lane is closed.

## Active Task

No active task remains in this workstream.

## File Scope

- `apps/android/app/src/main/java/dev/nako/android/ui/`
- `apps/android/app/src/main/java/dev/nako/android/ui/screens/settings/`
- `apps/android/app/src/main/java/dev/nako/android/ui/screens/player/`
- Workstream docs under this directory.

## Validation

Run:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.settings.* --tests dev.nako.android.ui.screens.player.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
git diff --check
```

## Notes

- Keep adapter small and plain-text only.
- Do not add user feedback in this cleanup lane.
- Do not touch generated `output/` or `tmp/`.
- Future copy actions should use `rememberNakoClipboard()` instead of importing
  Compose clipboard APIs directly in route files.
