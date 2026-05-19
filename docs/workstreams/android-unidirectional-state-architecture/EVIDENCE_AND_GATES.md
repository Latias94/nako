# Android Unidirectional State Architecture - Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Required Gates

- Focused JVM tests for each migrated `BrowseSession` slice.
- Existing navigation, playback start, player exit, and browse presentation
  tests continue passing.
- Full `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`.
- `git diff --check`.
- Smoke regression if runtime behavior changes beyond session wiring.

## Evidence Log

- 2026-05-19: Lane opened. Implementation gates pending.
- 2026-05-19: AUSA-020 moved browse navigation actions behind
  `BrowseSession.reduce`, added `BrowseShellStateSaver`, and wired
  `TaruBrowseShell` navigation callbacks through `BrowseAction`.
  Gate passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionTest --tests dev.taru.android.ui.browse.TaruRouteStackTest --tests dev.taru.android.ui.browse.TaruBrowseNavigationStateSaverTest --no-daemon --no-parallel`.
- 2026-05-19: AUSA-030 moved Home, Library Detail, Search, and Browse Facet
  loading behind `BrowseSession` and `BrowseDataSource`. `TaruBrowseShell`
  now renders session state for the migrated slice and dispatches explicit
  retry/search/route actions. Stale route and stale search responses are
  guarded by request generations.
  Gates passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionLoadingTest --no-daemon --no-parallel`;
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`.
