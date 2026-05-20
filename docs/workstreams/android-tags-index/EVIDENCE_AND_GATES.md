# Android Tags Index - Evidence And Gates

Status: Active
Last updated: 2026-05-20

## Gates

Workstream document gate:

```powershell
Get-Content -LiteralPath 'docs/workstreams/android-tags-index/WORKSTREAM.json' -Raw | ConvertFrom-Json | Out-Null
```

Focused client gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.browse.TaruBrowseClientTest --no-daemon
```

Focused browse UI gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests "dev.taru.android.ui.browse.*" --no-daemon
```

Broader Android unit gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
```

Smoke gate, if Tags gets a stable fixture assertion:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0
```

Diff hygiene:

```powershell
git diff --check
```

## Evidence

- ATI-010 pending.

## Notes

- Reuse the existing Tag related Media Items route.
- Do not filter cached item lists locally.
- Generated smoke evidence under `apps/android/build/` should not be
  committed.
