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

- Task ID: ARI-030
- Owner: unassigned
- Files:
  - `apps/android/app/src/main/java/dev/taru/android/ui/browse/`
  - focused session/navigation tests
- Validation:
  - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests "dev.taru.android.ui.browse.*" --no-daemon`
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
- Tags Index is accepted but waits for the Genre index shape.
- Top-level People Index is deferred until the app has a richer role/search IA.

## Blockers

- None for ARI-020. Public server list routes already exist; Android needs the
  typed client contract.

## Next Recommended Action

- Execute ARI-030: add Genre Index route state to `BrowseSession`, including
  open/save/restore/load/retry/back behavior and row actions into existing
  Genre related Media Items routes.
