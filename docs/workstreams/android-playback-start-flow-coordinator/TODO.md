# Android Playback Start Flow Coordinator - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Lane Setup

- [x] APSF-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-playback-start-flow-coordinator]
  Goal: Open the refactor lane and define the shell/coordinator boundary.
  Validation: workstream docs exist and agree.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Handoff: APSF-020 is ready.

## M1 - Coordinator Extraction

- [x] APSF-020 [owner=codex] [deps=APSF-010] [scope=apps/android/app/src/main/java/dev/nako/android/playback,apps/android/app/src/main/java/dev/nako/android/ui/browse,apps/android/app/src/test/java/dev/nako/android/playback]
  Goal: Extract playback start preflight, resume resolution, and launch request
  construction into a focused coordinator/use case.
  Validation: focused Android JVM tests prove Remux start preflight, missing
  token failure, and resume source propagation.
  Evidence: `PlaybackStartCoordinatorTest`, `PlaybackResumeResolverTest`, and
  adjacent playback regression tests passed on 2026-05-19.
  Handoff: APSF-030 completed in the same closeout pass.

## M2 - Closeout

- [x] APSF-030 [owner=codex] [deps=APSF-020] [scope=docs/workstreams/android-playback-start-flow-coordinator]
  Goal: Verify no behavior regression and close the workstream.
  Validation: targeted Android JVM tests, `git diff --check`.
  Evidence: closeout notes in `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
