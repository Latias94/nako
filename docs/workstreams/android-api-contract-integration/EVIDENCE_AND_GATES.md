# Android API Contract Integration - Evidence And Gates

Status: Closed
Last updated: 2026-05-20

## Gates

Workstream document gate:

```powershell
Get-Content -LiteralPath 'docs/workstreams/android-api-contract-integration/WORKSTREAM.json' -Raw | ConvertFrom-Json | Out-Null
```

Focused Android unit gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.browse.NakoBrowseClientTest --no-daemon
```

Broader Android unit gate:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
```

Smoke gate after UI implementation:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0
```

Diff hygiene:

```powershell
git diff --check
```

## Evidence

- APICI-010: Workstream opened on 2026-05-20 after reviewing
  `docs/api/HTTP_API.md`,
  `docs/workstreams/android-public-client-api-coverage/API_COVERAGE_MATRIX.md`,
  `ClientBrowseDataSource`, and `BrowseSession`.
- APICI-020: Focused browse client contract test passed on 2026-05-20:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.browse.NakoBrowseClientTest --no-daemon`.
  Android now has typed `GET /people/{person_id}` coverage through
  `NakoBrowseClient.personDetail`, `PersonResponse`, and `MissingPerson`
  diagnostics.
- APICI-020: Broader Android debug unit gate passed on 2026-05-20:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`.
- APICI-030: Focused UI browse test gate passed on 2026-05-20:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests "dev.nako.android.ui.browse.*" --no-daemon`.
  This proves Person Detail route save/restore, BrowseSession route loading,
  stale response rejection, and `ClientBrowseDataSource` request ordering for
  `GET /people/{person_id}` followed by
  `GET /people/{person_id}/items?limit=24&offset=0`.
- APICI-030: Broader Android debug unit gate passed on 2026-05-20:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`.
- APICI-040: Focused detail presentation test passed on 2026-05-20:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.screens.detail.MediaItemDetailRouteTest --no-daemon`.
  This proves stable Cast & Crew `person_id` rows produce Person Detail
  targets while missing IDs remain explicit Person facet/API-gap fallbacks.
- APICI-040: Broader Android debug unit gate passed on 2026-05-20:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`.
  `NakoBrowseShell` now renders `NakoRoute.PersonDetail` through a dedicated
  Person Detail screen instead of the temporary facet bridge.
- APICI-050: Focused server-backed smoke passed on 2026-05-20:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0`.
  Evidence directory:
  `apps/android/build/smoke-regression/20260520-125519/states/profile-with-media/20260520-125549-profile-with-media-emulator-5554/`.
  The `person-detail` criteria proves `Mira Vale`, `Person`, `1 related`,
  `Related Media Items`, and `Night Harbor` are visible after opening the
  Cast & Crew row.
- APICI-060: People, Tags, and Genres index pages were split on 2026-05-20 to
  `docs/workstreams/android-relationship-indexes/`. This closes the API
  contract lane without forcing broad index IA into the Person Detail proof.

## Notes

- Generated smoke reports, screenshots, UI dumps, and fixture data under
  `apps/android/build/` are evidence anchors only and should not be committed.
- Public route coverage should be tested at the Android client seam before UI
  work consumes it.
