# Android Navigation State Restoration - Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

ANS-010 through ANS-030 are complete. The lane is closed.

## Active Task

No active task remains in this workstream.

## File Scope

- `apps/android/app/src/main/java/dev/nako/android/ui/browse/`
- `apps/android/app/src/test/java/dev/nako/android/ui/browse/`
- Workstream docs under this directory.

## Validation

Run:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.browse.NakoBrowseNavigationStateSaverTest --no-daemon
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
git diff --check
```

Last passing evidence:

- Focused saver test: PASS on 2026-05-19.
- Android debug unit test suite: PASS on 2026-05-19.
- Diff hygiene: PASS on 2026-05-19.

## Notes

- Do not serialize `PlaybackLaunchRequest`.
- Player route restoration should fall back to the previous safe route.
- Do not adopt Jetpack Navigation in this lane.
- Do not touch untracked `output/` or `tmp/`.
- Follow-ons, if needed, should be opened separately for deep links, route URI
  contracts, or active playback session restoration.
