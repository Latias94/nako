# Android Player Route Host - Evidence And Gates

Status: Closed
Last updated: 2026-05-20

## Gates

Focused host gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.player.PlayerRouteHostTest --no-daemon --no-parallel
```

Focused player gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.player.* --no-daemon --no-parallel
```

Final gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel
git diff --check
```

## Evidence

- APRH-010: Workstream docs opened on 2026-05-20.
- APRH-020: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.player.PlayerRouteHostTest --no-daemon --no-parallel`
  passed on 2026-05-20. Proves the host owns prepare/retry, route engine
  callbacks, sanitized error state, idempotent attach/dispose/release, and exit
  effect triggering.
- APRH-030: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.player.* --no-daemon --no-parallel`
  passed on 2026-05-20. Proves the player package remains green after
  `PlaybackPlayerRoute` delegates lifecycle orchestration to `PlayerRouteHost`.
- APRH-040: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`
  passed on 2026-05-20.
- APRH-040: `git diff --check` passed on 2026-05-20. Output included only the
  existing CRLF normalization warning for `PlaybackPlayerRoute.kt`.
