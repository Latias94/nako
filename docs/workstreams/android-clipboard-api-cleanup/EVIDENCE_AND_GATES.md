# Android Clipboard API Cleanup - Evidence And Gates

Status: Closed
Last updated: 2026-05-20

## Gates

Focused settings/player gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.settings.* --tests dev.nako.android.ui.screens.player.* --no-daemon --no-parallel
```

Final gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
git diff --check
```

## Evidence

- ACAC-010: Local `ui-android:1.11.1` AAR inspected with `jar`/`javap`; new
  clipboard API confirmed.
- ACAC-020: `NakoClipboard` adapter added around `LocalClipboard`.
- ACAC-030: Settings/player diagnostics copy calls migrated to
  `copyPlainText`.
  - PASS:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.settings.* --tests dev.nako.android.ui.screens.player.* --no-daemon --no-parallel`
- ACAC-040: Final closeout gates passed on 2026-05-20.
  - PASS:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
  - PASS: `git diff --check`
