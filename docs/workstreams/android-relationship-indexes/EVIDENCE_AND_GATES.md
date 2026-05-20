# Android Relationship Indexes - Evidence And Gates

Status: Closed
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
- ARI-010 completed on 2026-05-20:
  - Genres Index accepted as the first implementation slice.
  - Tags Index accepted as a second reuse slice after Genres.
  - Top-level People Index deferred; Person Detail remains the primary People
    browsing path.
  - Android API integration matrix updated with the accepted/deferred status.
- ARI-020 completed on 2026-05-20:
  - Added `GenreListResponse` and `TaruBrowseClient.listGenres`.
  - Added focused client coverage for `GET /genres?limit=&offset=` request
    construction, bearer auth, response decoding, safe request redaction, and
    unsupported API version diagnostics.
  - Fresh validation:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.browse.TaruBrowseClientTest --no-daemon`
    passed.
- ARI-030 completed on 2026-05-20:
  - Added `RelationshipIndexFamily.Genres`, `TaruRoute.RelationshipIndex`, and
    `RelationshipIndexUiState`.
  - `BrowseSession` can open, save, restore, load, retry, and back out of the
    Genre Index route.
  - `ClientBrowseDataSource.loadRelationshipIndex` calls
    `TaruBrowseClient.listGenres` and maps Genre rows to existing
    server-backed Genre related Media Items route targets.
  - Fresh validation:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests "dev.taru.android.ui.browse.*" --no-daemon`
    passed.
- ARI-040 completed on 2026-05-20:
  - Added `RelationshipIndexRouteContent` with an artwork-led header, Genre
    row list, loading/failure/empty states, and stable row actions into
    existing Genre related Media Items routes.
  - Added the Home Genres anchor as a nested route entry point, not a new
    bottom navigation destination.
  - Added `RelationshipIndexRouteTest` for presentation counts and stable
    Genre targets.
  - Fresh validation:
    `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
    passed.
- ARI-050 completed on 2026-05-20:
  - Extended `profile-with-media` smoke to prove the Home Genres anchor,
    server-backed Genres Index, and Genre Index row into the existing Genre
    related Media Items route.
  - Smoke evidence directory:
    `apps/android/build/smoke-regression/20260520-141446/states/profile-with-media/20260520-141503-profile-with-media-emulator-5554/`.
  - Smoke surfaces include `genre-index.png` and
    `genre-index-facet.png`; the structured regression report is
    `apps/android/build/smoke-regression/20260520-141446/report.json`.
  - Fresh validation:
    `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0`
    passed.
  - Tags Index is split to `docs/workstreams/android-tags-index/`; top-level
    People Index remains deferred.

## Notes

- Reuse the existing Person Detail and Browse Facet related-items routes.
- Do not create local filtered indexes from cached item data.
- Generated smoke evidence under `apps/android/build/` should not be
  committed.
