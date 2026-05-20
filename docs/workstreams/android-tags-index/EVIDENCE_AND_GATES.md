# Android Tags Index - Evidence And Gates

Status: Closed
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
- ATI-030 completed on 2026-05-20:
  - Added the Home Tags anchor next to Genres as a nested relationship index
    entry point.
  - `TaruBrowseShell` dispatches the Home Tags anchor through
    `BrowseAction.OpenRelationshipIndex(RelationshipIndexFamily.Tags)`.
  - `RelationshipIndexRoute` keeps the shared Material Expressive screen shape
    and now uses family-aware copy and icons for Genres and Tags.
  - Added focused host coverage proving the Tags relationship index route is
    opened and loaded through the shared route loader.
  - Fresh validation:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseShellHostTest --no-daemon`
    passed.
  - Fresh validation:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseShellHostTest --tests dev.taru.android.ui.screens.relationship.RelationshipIndexRouteTest --no-daemon`
    passed.
  - Fresh validation:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
    passed.
- ATI-040 completed on 2026-05-20:
  - Added `profile-with-media` smoke coverage for Home -> Tags -> Lighthouse
    -> Related Media Items, reusing the existing relationship index and facet
    smoke helpers.
  - Smoke evidence captured `tag-index` and `tag-index-facet` surfaces.
  - Fresh validation:
    `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0`
    passed with zero retries.
  - Smoke regression report:
    `apps/android/build/smoke-regression/20260520-164905/report.md`.
  - Per-state smoke evidence:
    `apps/android/build/smoke-regression/20260520-164905/states/profile-with-media/20260520-164936-profile-with-media-emulator-5554/report.md`.
  - Note: the fixture server build emitted pre-existing Rust warnings in
    `taru-server` unused code paths; the smoke gate itself passed.

## Notes

- Reuse the existing Tag related Media Items route.
- Do not filter cached item lists locally.
- Generated smoke evidence under `apps/android/build/` should not be
  committed.
- This workstream is closed. Follow-ons should open separate lanes for
  CI/device-farm smoke execution, golden screenshot diffing, or richer Tags IA
  such as sorting and clustering.
