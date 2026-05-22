# Android Active Playback Session Cancellation - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Lane Setup

- [x] APSC-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-active-playback-session-cancellation]
  Goal: Open the follow-on lane and define the active cancellation evidence
  target.
  Validation: workstream docs exist and agree.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Handoff: First executable task is APSC-020.

## M1 - Active Remux Session Contract

- [x] APSC-020 [owner=codex] [deps=APSC-010] [scope=crates/nako-server/src/app/playback,crates/nako-server/src/http/playback.rs,crates/nako-server/src/http/tests/playback.rs]
  Goal: Provide a Public Client remux preflight/start path that returns a
  non-terminal remux session id before ffmpeg finishes and can be cancelled.
  Validation: focused nako-server HTTP test proves HEAD/start returns active
  session id and cancel readback reaches cancelled.
  Evidence: `cargo test -p nako-server remux_stream_route -- --nocapture`.
  Handoff: APSC-030 wired Android smoke after APSC-020 passed.

## M2 - Android Smoke Fixture Path

- [x] APSC-030 [owner=codex] [deps=APSC-020] [scope=apps/android/app/src/debug,apps/android/app/src/main/java/dev/nako/android/playback,apps/android/app/src/main/java/dev/nako/android/ui/browse,apps/android/scripts/Smoke-Emulator.ps1,apps/android/scripts/Start-DemoFixtureServer.ps1]
  Goal: Add a dedicated active-remux smoke state that forces non-Direct
  playback, starts player playback with a real session id, exits before
  completion, and reads back cancelled state.
  Validation: focused Android unit tests for fixture capabilities plus smoke
  command for the new state.
  Evidence: `apps/android/build/smoke/20260519-223623-profile-active-remux-emulator-5554/profile-active-remux-session-cancelled.txt`.
  Handoff: APSC-040 closed the lane after fresh verification.

## M3 - Closeout

- [x] APSC-040 [owner=codex] [deps=APSC-030] [scope=docs/workstreams/android-active-playback-session-cancellation,apps/android/SMOKE_FIXTURES.md,apps/android/README.md]
  Goal: Update docs, record gates, close or split remaining follow-ons.
  Validation: targeted gates plus `git diff --check`.
  Evidence: closeout notes in `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
