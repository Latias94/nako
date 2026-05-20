# Android Tags Index - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

This lane is the follow-on from the closed
`docs/workstreams/android-relationship-indexes/` lane. Genres Index proved the
relationship index shape end to end. Tags Index should reuse that shape as the
second accepted relationship index family. ATI-010 is complete:
`TagListResponse` and `TaruBrowseClient.listTags` cover
`GET /tags?limit=&offset=`.

## Active Task

- Task ID: ATI-020
- Owner: unassigned
- Files:
  - `apps/android/app/src/main/java/dev/taru/android/ui/browse/`
  - focused browse session/navigation/data-source tests.
- Validation:
  - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests "dev.taru.android.ui.browse.*" --no-daemon`
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

## Blockers

- None for ATI-020.

## Next Recommended Action

- Execute ATI-020: extend the relationship index route state and data-source
  mapping to Tags, reusing the proven Genres shape.
