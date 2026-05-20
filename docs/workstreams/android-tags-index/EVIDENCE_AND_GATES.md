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

- ATI-010 completed on 2026-05-20:
  - Added `TagListResponse` and `TaruBrowseClient.listTags`.
  - Added focused client coverage for `GET /tags?limit=&offset=` request
    construction, bearer auth, response decoding, safe request redaction, and
    unsupported API version diagnostics.
  - Fresh validation:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.browse.TaruBrowseClientTest --no-daemon`
    passed.
- ATI-020 completed on 2026-05-20:
  - Added `RelationshipIndexFamily.Tags` and
    `TagListResponse.toRelationshipIndexContent`.
  - `BrowseSession` opens and loads Tags Index through the existing
    relationship index route state; Tag rows open existing Tag related Media
    Items routes.
  - `ClientBrowseDataSource.loadRelationshipIndex(Tags)` calls
    `TaruBrowseClient.listTags` and maps rows to stable Tag facet targets.
  - Navigation save/restore preserves Tags Index as a safe nested route.
  - `RelationshipIndexRoute` presentation copy is family-aware for Genres and
    Tags.
  - Fresh validation:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.relationship.RelationshipIndexRouteTest --no-daemon --rerun-tasks`
    passed.
  - Fresh validation:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests "dev.taru.android.ui.browse.*" --no-daemon`
    passed.
  - Note: an earlier parallel Gradle run hit Kotlin incremental cache
    contention; rerunning the same gates serially passed.

## Notes

- Reuse the existing Tag related Media Items route.
- Do not filter cached item lists locally.
- Generated smoke evidence under `apps/android/build/` should not be
  committed.
