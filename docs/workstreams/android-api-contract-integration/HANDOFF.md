# Android API Contract Integration - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

APICI-010 through APICI-050 are complete. The matrix shows Android already has
real server-backed coverage for connection, browse, artwork, playback,
playback sessions, and User Playback State. Android now also has typed client
coverage and route/data-source state for `GET /people/{person_id}` plus
related Media Items. Person Detail now has a dedicated UI route, and stable
Cast & Crew `person_id` rows open that route instead of the generic facet
screen. Focused `profile-with-media` smoke proves that path against the real
fixture server.

## Active Task

APICI-060 - Decide People/Tags/Genres indexes.

## File Scope

Expected next files:

- `docs/workstreams/android-api-contract-integration/API_INTEGRATION_MATRIX.md`
- `docs/workstreams/android-api-contract-integration/DESIGN.md`
- `docs/workstreams/android-api-contract-integration/TODO.md`
- follow-on workstream docs if productizing index pages now

## Validation

Latest APICI-050 evidence:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0
```

For APICI-060, decide whether People, Tags, and Genres index pages belong in
this workstream or should become explicit follow-on lanes.

## Notes

- Do not consume Admin/internal server routes from Android.
- Keep the existing `TaruHttpTransport` seam.
- Person Detail client support exists as `TaruBrowseClient.personDetail`.
- Person Detail route state reuses the existing person-items route for related
  Media Items instead of duplicating response parsing.
- `TaruBrowseShell` no longer uses `PersonDetailUiState.toFacetState()`.
- `Smoke-Emulator.ps1` now captures `person-detail` instead of `facet-person`
  for the Cast & Crew path.
