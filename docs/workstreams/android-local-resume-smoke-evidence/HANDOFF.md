# Android Local Resume Smoke Evidence - Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

Lane is closed. ALR-010, ALR-020, and ALR-030 are complete.

## Completed Slice

ALR-020 allowed the debug smoke fixture to inject a device-local playback
position and made `profile-with-media` smoke prove the local resume UI path.

## File Scope

- `apps/android/app/src/debug/java/dev/nako/android/smoke/`
- `apps/android/app/src/testDebug/java/dev/nako/android/smoke/`
- `apps/android/scripts/Smoke-Emulator.ps1`
- Workstream docs under this directory.

## Validation

Passed:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.smoke.DebugSmokeFixtureSeedActivityTest --no-daemon
pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media
git diff --check
```

Latest focused smoke report:

`apps/android/build/smoke/20260519-102517-profile-with-media-emulator-5554/report.md`

Latest regression smoke report:

`apps/android/build/smoke-regression/20260519-102943/report.md`

## Notes

- The seed path remains debug-only.
- No Public Client API or server changes were introduced in this lane.
- Device-local resume still does not claim cross-device **User Playback State**.
- Generated smoke evidence under `apps/android/build` is not committed.
- Follow-ons: CI/device-farm execution, golden screenshot diffing, and deeper
  playback duration/seek validation.
