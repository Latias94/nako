# Android Public Client API Coverage Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Coverage Baseline

Exit criteria:

- The current Public Client API v1 route set is mapped to Android coverage.
- Each gap has one of: implement next, defer, split server contract, or non-goal.
- The route matrix names Android owners for covered routes.

Evidence:

- `API_COVERAGE_MATRIX.md`
- `TODO.md` APIC-010 complete

## M1 - Selected Artwork Image Slice

Exit criteria:

- Android can build authenticated selected artwork image requests for public
  `/images/{image_id}` routes.
- Browse/detail surfaces prefer real selected artwork when available.
- Player remains video-first and uses deterministic local fallback rather than
  carrying authenticated artwork image requests into playback state.
- Missing artwork falls back to deterministic placeholders.
- Token values are not logged, displayed, or stored in unsafe request previews.

Evidence:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
- `pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States profile-with-media`
- `TODO.md` APIC-020 and APIC-030 complete

## M2 - Route Gap Decisions

Exit criteria:

- Library detail/source inventory is implemented.
- Direct source probe route consumption is implemented.
- People/tag/genre list/detail routes remain a deliberate product backlog item,
  not an accidental omission.

Evidence:

- `TaruBrowseClient.libraryDetail` and `TaruBrowseClient.librarySources`
- `LibraryDetailRouteContent`
- `TaruPlaybackClient.getSourceProbe` and Source Picker source facts
- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
- `TODO.md` APIC-040 and APIC-050 complete

## M3 - Playback State Contract Decision

Exit criteria:

- Android's device-local resume boundary remains explicit.
- Cross-device Continue Watching is split into a server/client User Playback
  State workstream.

Evidence:

- `docs/workstreams/user-playback-state-contract/`
- `TODO.md` APIC-060 complete

## M4 - Closeout

Exit criteria:

- All implemented Android API clients have focused tests.
- `API_COVERAGE_MATRIX.md` matches the current `docs/api/HTTP_API.md` public
  route inventory.
- Remaining work is split into named follow-ons or explicitly deferred.

Evidence:

- `apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon`
- `git diff --check`
- Public route inventory check: 27 routes matched in `docs/api/HTTP_API.md`
- `TODO.md` APIC-070 complete
