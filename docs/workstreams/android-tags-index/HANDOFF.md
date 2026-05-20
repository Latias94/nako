# Android Tags Index - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

This lane is the follow-on from the closed
`docs/workstreams/android-relationship-indexes/` lane. Genres Index proved the
relationship index shape end to end. Tags Index should reuse that shape as the
second accepted relationship index family.

## Active Task

- Task ID: ATI-010
- Owner: unassigned
- Files:
  - `apps/android/app/src/main/java/dev/taru/android/browse/`
  - `apps/android/app/src/test/java/dev/taru/android/browse/TaruBrowseClientTest.kt`
  - API coverage matrices.
- Validation:
  - `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.browse.TaruBrowseClientTest --no-daemon`
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

## Blockers

- None for ATI-010.

## Next Recommended Action

- Execute ATI-010: add typed Android client coverage for
  `GET /tags?limit=&offset=`.
