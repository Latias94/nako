# Android Unidirectional State Architecture - Evidence And Gates

Status: Closed
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
- 2026-05-19: AUSA-040 moved Media Item Detail, selected Media Source,
  source probe, playback decision, and retry state into `BrowseSession`.
  `ClientBrowseDataSource` now owns token/client access for those loads.
  `TaruBrowseShell` renders the session state and keeps only playback-start
  orchestration for AUSA-050. Token-missing detail/probe/decision paths return
  safe failure states without issuing HTTP requests.
  Gates passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionLoadingTest --tests dev.taru.android.ui.browse.ClientBrowseDataSourceTest --tests dev.taru.android.ui.browse.BrowseSessionTest --no-daemon --no-parallel`;
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`.
- 2026-05-19: AUSA-050 moved playback start orchestration into
  `BrowseSession` through `BrowsePlaybackStarter` and
  `ClientBrowsePlaybackStarter`. `TaruBrowseShell` now dispatches
  `BrowseAction.StartPlayback` and no longer builds `PlaybackStartRequest` or
  launches the start coroutine. Success opens the Player route; failure stores
  playback diagnostics in session state.
  Gates passed:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionLoadingTest --tests dev.taru.android.ui.browse.BrowseSessionTest --no-daemon --no-parallel`;
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel`.
- 2026-05-19: AUSA-060 moved resume-position calculation behind
  `BrowseResumeResolver`, leaving `TaruBrowseShell` as a Compose adapter that
  creates production adapters, observes `BrowseSession.state`, renders routes,
  and dispatches `BrowseAction`. Targeted and full debug unit gates passed.
- 2026-05-19: AUSA-070 closeout completed. Final gate:
  `git diff --check`. Smoke regression was not rerun because this lane changed
  internal Android state orchestration and preserved existing public client and
  playback semantics under JVM coverage.
