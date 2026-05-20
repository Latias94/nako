# Android Browse Shell Session Host - Evidence And Gates

Status: Closed
Last updated: 2026-05-20

## Gates

Focused host gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseShellHostTest --no-daemon --no-parallel
```

Focused browse gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon --no-parallel
```

Final gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
git diff --check
```

## Evidence

- ABSH-010: Workstream docs opened on 2026-05-20.
- ABSH-020: `BrowseShellHost` and focused host tests added on 2026-05-20.
  - PASS:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseShellHostTest --no-daemon --no-parallel`
- ABSH-030: `TaruBrowseShell` now delegates host lifecycle and client runtime
  assembly to `BrowseShellHost` / `ClientBrowseShellRuntime`.
  - PASS:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --no-daemon --no-parallel`
- ABSH-040: Final closeout gates passed on 2026-05-20.
  - PASS:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
  - PASS: `git diff --check`
