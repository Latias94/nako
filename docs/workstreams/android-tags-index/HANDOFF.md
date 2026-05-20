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
restore path. ATI-030 is complete: Home exposes Tags as a nested relationship
index route, `TaruBrowseShell` dispatches it through the shared Tags
relationship index action, and `RelationshipIndexRoute` keeps one screen shape
with family-aware copy and icons.

## Active Task

- Task ID: ATI-040
- Owner: Codex
- Files:
  - `apps/android/scripts/Smoke-Emulator.ps1`
  - `apps/android/scripts/Smoke-Regression.ps1`
  - `docs/workstreams/android-tags-index/`
- Validation:
  - `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -RetriesPerState 0`
  - `git diff --check`
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
- Tags now has a Home anchor and relationship index icon parity with Genres.

## Blockers

- None for ATI-040.

## Next Recommended Action

- Execute ATI-040: decide whether the existing profile-with-media smoke fixture
  can prove Home -> Tags -> related Media Items. If the fixture is not stable
  or does not expose Tags through UIAutomator-accessible labels, record an
  explicit non-smoke rationale and close the lane with fresh unit evidence.
