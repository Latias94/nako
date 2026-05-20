# Android API Contract Integration - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

APICI-010 through APICI-040 are complete. The matrix shows Android already has
real server-backed coverage for connection, browse, artwork, playback,
playback sessions, and User Playback State. Android now also has typed client
coverage and route/data-source state for `GET /people/{person_id}` plus
related Media Items. Person Detail now has a dedicated UI route, and stable
Cast & Crew `person_id` rows open that route instead of the generic facet
screen.

## Active Task

APICI-050 - Prove server-backed Person Detail smoke.

## File Scope

Expected next files:

- `apps/android/scripts/Smoke-Emulator.ps1`
- `apps/android/scripts/Smoke-Regression.ps1`
- smoke fixture docs if the assertion path changes

## Validation

Latest APICI-040 evidence:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.detail.MediaItemDetailRouteTest --no-daemon
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
```

For APICI-050, update the `profile-with-media` smoke path so Cast & Crew opens
the Person Detail route, then returns through related Media Items.

## Notes

- Do not consume Admin/internal server routes from Android.
- Keep the existing `TaruHttpTransport` seam.
- Person Detail client support exists as `TaruBrowseClient.personDetail`.
- Person Detail route state reuses the existing person-items route for related
  Media Items instead of duplicating response parsing.
- `TaruBrowseShell` no longer uses `PersonDetailUiState.toFacetState()`.
