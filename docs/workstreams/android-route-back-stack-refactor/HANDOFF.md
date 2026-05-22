# Android Route Back-Stack Refactor - Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

ARB-010 through ARB-040 are complete. The lane is closed.

## Active Task

No active task remains in this workstream.

## File Scope

- `apps/android/app/src/main/java/dev/nako/android/ui/browse/`
- `apps/android/app/src/test/java/dev/nako/android/ui/browse/`
- `apps/android/scripts/Smoke-Emulator.ps1`
- `apps/android/SMOKE_FIXTURES.md`
- Workstream docs under this directory.

## Validation

Run:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.browse.NakoRouteStackTest --no-daemon
pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media -SkipBuild
git diff --check
```

Last passing evidence:

- Focused JVM test: PASS on 2026-05-19.
- `profile-with-media` smoke:
  `apps/android/build/smoke/20260519-112014-profile-with-media-emulator-5554/report.md`.
- `profile-missing-token` smoke:
  `apps/android/build/smoke/20260519-112354-profile-missing-token-emulator-5554/report.md`.
- Three-state regression:
  `apps/android/build/smoke-regression/20260519-112540/report.md`.

## Notes

- `NakoBrowseShell` now uses `NakoBrowseNavigationState` instead of a single
  overwrite-only `NakoRoute`.
- Smoke no longer compensates for facet Back by reopening detail from Home.
- Do not adopt Jetpack Navigation in this lane.
- Do not touch untracked `output/` or `tmp/`.
- Do not commit generated smoke evidence under `apps/android/build`.
- Follow-ons, if needed, should be opened separately for deep links,
  serialized routes, and process-death restoration.
