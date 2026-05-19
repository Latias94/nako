# Android Presentation Runtime Adapters - Evidence And Gates

Status: Closed
Last updated: 2026-05-20

## Gate Plan

Primary final gates:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
git diff --check
```

Focused gates by task:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.artwork.* --tests dev.taru.android.ui.browse.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionLoadingTest --tests dev.taru.android.ui.artwork.* --no-daemon --no-parallel
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.* --tests dev.taru.android.player.* --no-daemon --no-parallel
```

## Evidence Log

| Task | Status | Evidence |
| --- | --- | --- |
| APRA-010 | Done | Lane docs created and aligned with player-session follow-on. |
| APRA-020 | Done | `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.artwork.* --tests dev.taru.android.ui.browse.* --no-daemon --no-parallel` passed on 2026-05-20. |
| APRA-030 | Done | Detail visual APIs now receive `ArtworkRequestResolver`; focused JVM gate passed on 2026-05-20. |
| APRA-040 | Done | `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.artwork.* --tests dev.taru.android.ui.browse.* --tests dev.taru.android.player.* --no-daemon --no-parallel` passed on 2026-05-20. |
| APRA-050 | Done | `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel` and `git diff --check` passed on 2026-05-20. |

## Notes

- Emulator smoke is not a default gate for this lane because this is an
  internal presentation/runtime contract refactor. Add smoke only if player
  launch or artwork behavior changes beyond adapter boundaries.
- Do not commit generated evidence under `apps/android/build/`.
