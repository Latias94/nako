# Android Relationship Indexes - Evidence And Gates

Status: Active
Last updated: 2026-05-20

## Gates

Workstream document gate:

```powershell
Get-Content -LiteralPath 'docs/workstreams/android-relationship-indexes/WORKSTREAM.json' -Raw | ConvertFrom-Json | Out-Null
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

Smoke gate, if an accepted index path reaches the fixture:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0
```

Diff hygiene:

```powershell
git diff --check
```

## Evidence

- ARI-010 will record the product decision for People, Tags, and Genres index
  pages before implementation begins.

## Notes

- Reuse the existing Person Detail and Browse Facet related-items routes.
- Do not create local filtered indexes from cached item data.
- Generated smoke evidence under `apps/android/build/` should not be
  committed.
