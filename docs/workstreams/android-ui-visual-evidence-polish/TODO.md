# Android UI Visual Evidence Polish - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Boundary Freeze

- [x] AUP-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-ui-visual-evidence-polish]
  Goal: Open a screenshot-driven polish lane and freeze scope to the Player
  chrome overlap found in smoke evidence.
  Validation: Workstream docs exist and agree.
  Evidence: `DESIGN.md`.
  Handoff: Completed on 2026-05-19.

## M1 - Player Chrome Overlap Polish

- [x] AUP-020 [owner=codex] [deps=AUP-010] [scope=apps/android/app/src/main/java/dev/taru/android/ui/screens/player,apps/android/app/src/test/java/dev/taru/android/ui/screens/player]
  Goal: Adjust Player custom chrome so it avoids Media3 controls while
  preserving context labels and standard playback controls.
  Validation:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.ui.screens.player.PlayerPresentationTest --no-daemon`
  plus `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media -SkipBuild` after compile.
  Review: Confirm the fix does not disable Media3 controls or introduce a custom
  playback controller.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: Completed on 2026-05-19. Added an explicit Media3 controller
  clearance for Taru Player context chrome and focused test coverage for the
  layout policy.

## M2 - Closeout

- [x] AUP-030 [owner=planner] [deps=AUP-020] [scope=docs/workstreams/android-ui-visual-evidence-polish]
  Goal: Verify evidence, close this lane, and defer broader Home/Detail visual
  ambitions.
  Validation: fresh AUP-020 gate plus `git diff --check`.
  Review: Confirm generated smoke evidence is not committed.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Closed on 2026-05-19 after focused player test, focused smoke, fresh
  screenshot review, and diff hygiene passed.
