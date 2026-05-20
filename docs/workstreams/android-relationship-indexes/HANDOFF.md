# Android Relationship Indexes - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

This lane was split from APICI-060 after Android API Contract Integration
completed the Person Detail route and smoke proof. ARI-010 is complete:
Genres Index is the first implementation slice, Tags Index follows as the
second reuse slice, and top-level People Index is deferred while Person Detail
remains the primary People path.

## Active Task

- Task ID: ARI-050
- Owner: unassigned
- Files:
  - `apps/android/scripts/Smoke-Emulator.ps1`
  - `docs/workstreams/android-relationship-indexes/`
- Validation:
  - `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0`
- Status: READY
- Review: pending
- Evidence: pending

## Decisions Since Last Update

- Person Detail belongs to the completed API contract lane.
- People, Tags, and Genres indexes are a separate product navigation lane.
- Existing related-items routes should be reused; no local filtering.
- Genres Index is the first slice and should open as a nested route from Home,
  not as a new bottom navigation destination.
- `TaruBrowseClient.listGenres` is the Android typed contract for
  `GET /genres?limit=&offset=`.
- `TaruRoute.RelationshipIndex(RelationshipIndexFamily.Genres)` and
  `RelationshipIndexUiState` are in place.
- `RelationshipIndexRouteContent` replaced the temporary placeholder and Home
  now exposes a Genres anchor into the nested route.
- Tags Index is accepted but waits for the Genre index shape.
- Top-level People Index is deferred until the app has a richer role/search IA.

## Blockers

- None for ARI-050. The next decision is whether to extend smoke assertions for
  the new Genres path now or record a short non-smoke closeout rationale.

## Next Recommended Action

- Execute ARI-050: prove the Genres Index route with smoke if practical, then
  close this lane or split Tags Index as the reuse follow-on.
