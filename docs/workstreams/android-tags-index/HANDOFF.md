# Android Tags Index - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

This lane is the follow-on from the closed
`docs/workstreams/android-relationship-indexes/` lane. Genres Index proved the
relationship index shape end to end. Tags Index should reuse that shape as the
second accepted relationship index family. ATI-010 is complete:
`TagListResponse` and `TaruBrowseClient.listTags` cover
`GET /tags?limit=&offset=`. ATI-020 is complete: Tags now reuse the
relationship index route state, data-source mapping, and safe navigation
restore path.

## Active Task

- Task ID: ATI-030
- Owner: unassigned
- Files:
  - relationship index screen family.
  - `HomeScreen` and `TaruBrowseShell`.
  - focused presentation tests where practical.
- Validation:
  - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
- Status: READY
- Review: pending
- Evidence: pending

## Decisions

- Tags Index is accepted as a reuse slice after Genres.
- Tags Index should open as a nested Home route, not a new bottom navigation
  destination.
- Tag rows should open existing Tag related Media Items routes with stable
  server IDs.
- Top-level People Index remains outside this lane.
- `TaruBrowseClient.listTags` is the Android typed contract for
  `GET /tags?limit=&offset=`.
- `RelationshipIndexFamily.Tags` is in place and maps to existing Tag related
  Media Items targets.
- Relationship index presentation copy is family-aware for Genres and Tags.

## Blockers

- None for ATI-030.

## Next Recommended Action

- Execute ATI-030: add the Home Tags entry point and ensure the shared
  relationship index screen works as the user-facing Tags Index route.
