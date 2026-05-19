# Android Unidirectional State Architecture - TODO

Status: Active
Last updated: 2026-05-19

## M0 - Lane Setup

- [x] AUSA-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-unidirectional-state-architecture]
  Goal: Open the UDF refactor lane and define the session/UI boundary.
  Validation: workstream docs exist and agree.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Handoff: AUSA-020 is ready.

## M1 - Browse Session Skeleton

- [x] AUSA-020 [owner=codex] [deps=AUSA-010] [scope=apps/android/app/src/main/java/dev/taru/android/ui/browse,apps/android/app/src/test/java/dev/taru/android/ui/browse]
  Goal: Add `BrowseShellState`, `BrowseAction`, and `BrowseSession` skeleton
  for navigation and selected destination state.
  Validation: JVM tests prove top-level destination selection, item/detail
  route opening, facet route opening, and back navigation through the session.
  Evidence: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionTest --tests dev.taru.android.ui.browse.TaruRouteStackTest --tests dev.taru.android.ui.browse.TaruBrowseNavigationStateSaverTest --no-daemon --no-parallel` passed on 2026-05-19.
  Handoff: AUSA-030 can migrate first async loading slice.

## M2 - First Async Loading Slice

- [x] AUSA-030 [owner=codex] [deps=AUSA-020] [scope=apps/android/app/src/main/java/dev/taru/android/ui/browse,apps/android/app/src/test/java/dev/taru/android/ui/browse]
  Goal: Move Home, Library Detail, Search, and Browse Facet loading actions
  from `TaruBrowseShell` into `BrowseSession`.
  Validation: JVM tests prove loading, success, failure, retry, and stale
  response handling for the migrated actions.
  Evidence: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionLoadingTest --no-daemon --no-parallel` and full `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel` passed on 2026-05-19.
  Handoff: AUSA-040 can migrate detail/source/playback selection.

## M3 - Detail And Playback Selection Slice

- [x] AUSA-040 [owner=codex] [deps=AUSA-030] [scope=apps/android/app/src/main/java/dev/taru/android/ui/browse,apps/android/app/src/test/java/dev/taru/android/ui/browse]
  Goal: Move Media Item Detail loading, selected Media Source state, source
  probe, playback decision, and retry events into `BrowseSession`.
  Validation: JVM tests prove item detail loading, selected source reset,
  source probe state, playback decision state, and token-safe failures.
  Evidence: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionLoadingTest --tests dev.taru.android.ui.browse.ClientBrowseDataSourceTest --tests dev.taru.android.ui.browse.BrowseSessionTest --no-daemon --no-parallel` and full `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel` passed on 2026-05-19.
  Handoff: AUSA-050 can migrate playback start route opening.

## M4 - Playback Start Integration

- [x] AUSA-050 [owner=codex] [deps=AUSA-040] [scope=apps/android/app/src/main/java/dev/taru/android/ui/browse,apps/android/app/src/main/java/dev/taru/android/playback,apps/android/app/src/test/java/dev/taru/android/ui/browse]
  Goal: Move playback start action handling into `BrowseSession` using
  `PlaybackStartCoordinator`, and make route opening a session state/effect.
  Validation: JVM tests prove Remux start preflight path still opens the player
  route and failure keeps playback diagnostics in state.
  Evidence: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.browse.BrowseSessionLoadingTest --tests dev.taru.android.ui.browse.BrowseSessionTest --no-daemon --no-parallel` and full `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon --no-parallel` passed on 2026-05-19.
  Handoff: AUSA-060 can remove local Compose orchestration.

## M5 - Compose Shell Cleanup

- [ ] AUSA-060 [owner=codex] [deps=AUSA-050] [scope=apps/android/app/src/main/java/dev/taru/android/ui/browse,apps/android/app/src/main/java/dev/taru/android/ui]
  Goal: Reduce `TaruBrowseShell` to state rendering and action dispatch.
  Validation: full Android debug unit tests, targeted browse/session tests, and
  smoke regression if runtime behavior changed.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: AUSA-070 can close after verification.

## M6 - Closeout

- [ ] AUSA-070 [owner=codex] [deps=AUSA-060] [scope=docs/workstreams/android-unidirectional-state-architecture]
  Goal: Verify the UDF architecture, record evidence, and close the workstream.
  Validation: targeted Android JVM tests, full `:app:testDebugUnitTest`,
  `git diff --check`, and smoke regression if needed.
  Evidence: closeout notes in `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
