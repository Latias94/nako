# Android Route Back-Stack Refactor - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Boundary Freeze

- [x] ARB-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-route-back-stack-refactor]
  Goal: Open the route back-stack refactor lane and freeze the scope to Android
  browse shell navigation behavior.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/android-route-back-stack-refactor/DESIGN.md`
  Handoff: Completed on 2026-05-19. First implementation slice should create a
  tested route stack model and wire it into `NakoBrowseShell`.

## M1 - Route Stack Model And Shell Wiring

- [x] ARB-020 [owner=codex] [deps=ARB-010] [scope=apps/android/app/src/main/java/dev/nako/android/ui/browse,apps/android/app/src/test/java/dev/nako/android/ui/browse]
  Goal: Replace single-route overwrite behavior with a tested route back-stack
  and wire detail, facet, player, settings, and root transitions through it.
  Validation:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.ui.browse.NakoRouteStackTest --no-daemon`
  plus Android local validation as risk requires.
  Review: Confirm route callers no longer encode their own return targets.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Completed on 2026-05-19. Implemented `NakoBrowseNavigationState`
  and `NakoRouteStack`, wired `NakoBrowseShell` through open/back/destination
  selection, and added focused JVM coverage.

## M2 - Smoke Return-Path Evidence

- [x] ARB-030 [owner=codex] [deps=ARB-020] [scope=apps/android/scripts,apps/android/SMOKE_FIXTURES.md]
  Goal: Update `profile-with-media` smoke to use and prove context-preserving
  Back from a detail-opened facet.
  Validation:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`
  plus
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media -SkipBuild`
  and `git diff --check`.
  Review: Confirm smoke no longer compensates for facet Back by reopening
  detail from Home.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Completed on 2026-05-19. Smoke no longer reopens detail from Home
  after facet Back; it asserts Detail after facet/player Back and Settings
  after Server Profile Back.

## M3 - Closeout

- [x] ARB-040 [owner=planner] [deps=ARB-030] [scope=docs/workstreams/android-route-back-stack-refactor]
  Goal: Verify evidence, close this lane, and split deeper navigation follow-ons.
  Validation: fresh ARB-020/ARB-030 gates and closeout doc updates.
  Review: Use review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Closed on 2026-05-19 after focused tests, device smoke, regression
  smoke, and diff hygiene passed.
