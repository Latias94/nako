# Android Browse Shell Session Host - Handoff

Status: Closed
Last updated: 2026-05-20

## Current State

ABSH-010 through ABSH-040 are complete. The lane is closed.

## Active Task

No active task remains in this workstream.

## File Scope

- `apps/android/app/src/main/java/dev/nako/android/ui/browse/`
- `apps/android/app/src/test/java/dev/nako/android/ui/browse/`
- Workstream docs under this directory.

## Validation

Run:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.browse.BrowseShellHostTest --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.browse.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
git diff --check
```

## Notes

- Keep `BrowseSession` as the state machine.
- Keep `BrowseShellHost` as lifecycle and integration orchestration.
- Do not touch generated `output/` or `tmp/`.
- `NakoBrowseShell` should stay a rendering shell. Put future browse lifecycle,
  route-displayed, and saveable-state orchestration behind `BrowseShellHost` or
  a successor host module.
