# Android Root App Composition - Evidence And Gates

Status: Closed
Last updated: 2026-05-20

## Gates

Focused root composition gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.NakoAppCompositionTest --no-daemon --no-parallel
```

Focused root UI gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.* --no-daemon --no-parallel
```

Final gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
git diff --check
```

## Evidence

- ARAC-010: Workstream docs opened on 2026-05-20.
- ARAC-020: `NakoAppEnvironment`, `AndroidNakoAppEnvironmentFactory`, and
  focused composition tests added.
  - PASS:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.NakoAppCompositionTest --no-daemon --no-parallel`
- ARAC-030: `NakoAndroidAppContent` now renders from root environment and app
  session instead of individual clients/stores.
  - PASS:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.* --no-daemon --no-parallel`
- ARAC-040: Final closeout gates passed on 2026-05-20.
  - PASS:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
  - PASS: `git diff --check`
