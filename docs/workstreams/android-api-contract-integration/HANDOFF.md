# Android API Contract Integration - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

APICI-010 through APICI-030 are complete. The matrix shows Android already has
real server-backed coverage for connection, browse, artwork, playback,
playback sessions, and User Playback State. Android now also has typed client
coverage and route/data-source state for `GET /people/{person_id}` plus
related Media Items.

## Active Task

APICI-040 - Build Person Detail screen.

## File Scope

Expected next files:

- `apps/android/app/src/main/java/dev/taru/android/ui/screens/person/`
- `apps/android/app/src/main/java/dev/taru/android/ui/browse/TaruBrowseShell.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/screens/detail/MediaItemDetailRoute.kt`
- focused UI/session tests

## Validation

Latest APICI-030 evidence:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests "dev.taru.android.ui.browse.*" --no-daemon
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
```

For APICI-040, replace the temporary `PersonDetailUiState.toFacetState()`
rendering in `TaruBrowseShell` with a dedicated screen.

## Notes

- Do not consume Admin/internal server routes from Android.
- Keep the existing `TaruHttpTransport` seam.
- Person Detail client support exists as `TaruBrowseClient.personDetail`.
- Person Detail route state reuses the existing person-items route for related
  Media Items instead of duplicating response parsing.
- `TaruBrowseShell` currently renders Person Detail through existing facet
  results as a temporary APICI-030 bridge; APICI-040 should remove that bridge.
