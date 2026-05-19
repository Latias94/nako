# Android Server-Backed Demo Fixtures — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

The workstream is closed. `ASD-010` completed the scope and evidence freeze.
`ASD-020` completed the route matrix and fixture strategy discovery. `ASD-030`
completed the first seeded server-backed fixture provider. `ASD-040` completed
the first Android media smoke state. `ASD-050` verified and closed the lane.

The key boundary is fixed: Android must not fake server-backed media fixture
data. Home, detail, source picker, and player smoke evidence must come through
Public Client API route shapes or an explicit local harness that implements
those public route shapes.

The first implementation strategy is now implemented by
`apps/android/scripts/Start-DemoFixtureServer.ps1`. It prepares a generated
Movies library with `Night Harbor`, runs real `taru-server scan` and
`import-nfo`, and can start a loopback server for Android access through
`adb reverse`.

The Android smoke flow is now implemented by
`apps/android/scripts/Smoke-Emulator.ps1 -FixtureState profile-with-media`. It
prepares and starts the fixture provider, applies `adb reverse`, seeds a
debug-only Server Profile and encrypted token value through the app's real
stores, then captures Home, detail, source picker, and player evidence.

## Final Task

- Task ID: ASD-050
- Owner: planner
- Files: `docs/workstreams/android-server-backed-demo-fixtures`, `apps/android`
- Validation: fresh Android unit tests, debug assemble, server fixture prepare,
  media smoke, and `git diff --check`
- Status: DONE
- Review: no blocking findings
- Evidence: recorded in `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Use `android-server-backed-demo-fixtures` as the durable lane for
  `profile-with-media` and `playback-ready` follow-ons from the Android QA
  harness.
- Treat the Public Client API as the only acceptable media fixture boundary for
  Android.
- Use a real seeded local `taru-server` as the first fixture provider strategy.
- Align Android `ClientTranscodePlan` with the Public Client API by removing the
  required internal `input_locator` field.
- Add a debug-only Android fixture writer instead of trying to hand-write
  encrypted token vault files from adb shell.
- Keep `profile-with-media` scoped to direct-play MP4 and player-safe launch.
- Keep CI, golden visual diffing, and deeper playback runtime validation out of
  the initial lane.

## Blockers

- No active blockers.

## Follow-Ons

- Add CI/device-farm integration for Android smoke when the project is ready for
  emulator infrastructure.
- Add golden visual diffing only after the V2 Material 3 Expressive UI surfaces
  stabilize.
- Add deeper HLS/remux/session and longer playback quality validation in a
  playback-focused lane.
