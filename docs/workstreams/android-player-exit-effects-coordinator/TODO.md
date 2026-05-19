# Android Player Exit Effects Coordinator - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Lane Setup

- [x] APEC-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-player-exit-effects-coordinator]
  Goal: Open the refactor lane and define the PlayerRoute/coordinator boundary.
  Validation: workstream docs exist and agree.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Handoff: APEC-020 is ready.

## M1 - Exit Coordinator Extraction

- [x] APEC-020 [owner=codex] [deps=APEC-010] [scope=apps/android/app/src/main/java/dev/taru/android/player,apps/android/app/src/main/java/dev/taru/android/ui/screens/player,apps/android/app/src/test/java/dev/taru/android/player]
  Goal: Extract exit-effect client wiring from `PlaybackPlayerRoute` into a
  tested coordinator/use case.
  Validation: focused Android JVM tests prove unfinished session cancellation,
  watched reporting, and missing-token local-only behavior through the
  coordinator.
  Evidence: `PlaybackExitCoordinatorTest` and adjacent playback/user playback
  tests passed on 2026-05-19.
  Handoff: APEC-030 completed in the same closeout pass.

## M2 - Closeout

- [x] APEC-030 [owner=codex] [deps=APEC-020] [scope=docs/workstreams/android-player-exit-effects-coordinator]
  Goal: Verify no behavior regression and close the workstream.
  Validation: targeted Android JVM tests, full debug unit tests when practical,
  `git diff --check`.
  Evidence: closeout notes in `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
