# Android API Contract Integration - Handoff

Status: Closed
Last updated: 2026-05-20

## Current State

APICI-010 through APICI-060 are complete. The matrix shows Android already has
real server-backed coverage for connection, browse, artwork, playback,
playback sessions, and User Playback State. Android now also has typed client
coverage and route/data-source state for `GET /people/{person_id}` plus
related Media Items. Person Detail now has a dedicated UI route, and stable
Cast & Crew `person_id` rows open that route instead of the generic facet
screen. Focused `profile-with-media` smoke proves that path against the real
fixture server. People, Tags, and Genres index pages are split to
`docs/workstreams/android-relationship-indexes/`.

## Active Task

None. This workstream is closed.

## File Scope

Follow-on files:

- `docs/workstreams/android-api-contract-integration/API_INTEGRATION_MATRIX.md`
- `docs/workstreams/android-relationship-indexes/`

## Validation

Latest APICI-050 evidence:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0
```

For relationship index work, continue with
`docs/workstreams/android-relationship-indexes/`.

## Notes

- Do not consume Admin/internal server routes from Android.
- Keep the existing `NakoHttpTransport` seam.
- Person Detail client support exists as `NakoBrowseClient.personDetail`.
- Person Detail route state reuses the existing person-items route for related
  Media Items instead of duplicating response parsing.
- `NakoBrowseShell` no longer uses `PersonDetailUiState.toFacetState()`.
- `Smoke-Emulator.ps1` now captures `person-detail` instead of `facet-person`
  for the Cast & Crew path.
- APICI-060 split index pages instead of productizing them inside this lane.
