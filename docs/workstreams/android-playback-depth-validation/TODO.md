# Android Playback Depth Validation - TODO

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Gate Freeze

- [x] APDV-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-playback-depth-validation]
  Goal: Open the lane and freeze the first Direct Play depth validation target.
  Validation: workstream docs exist and agree.
  Review: confirm HLS/remux/golden/CI remain out of the first slice.
  Evidence: `DESIGN.md`, `TODO.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`.
  Handoff: First executable task is APDV-020.

## M1 - Direct Play Advancement Smoke

- [ ] APDV-020 [owner=codex] [deps=APDV-010] [scope=apps/android/scripts/Smoke-Emulator.ps1,apps/android/SMOKE_FIXTURES.md,docs/workstreams/android-playback-depth-validation]
  Goal: Extend `profile-with-media` smoke evidence so Direct Play proves
  playback advanced beyond the seeded server resume point.
  Validation: `pwsh -NoProfile -File apps/android/scripts/Smoke-Emulator.ps1 -FixtureState profile-with-media -SkipAppBuild -SkipFixtureServerBuild`; parse/check generated evidence; `git diff --check`.
  Review: keep the check deterministic and avoid adding sleeps without an
  observable condition.
  Evidence: `EVIDENCE_AND_GATES.md`, generated smoke evidence path.
  Handoff: APDV-030 can verify server readback after player exit.

## M2 - Server Progress Readback After Exit

- [ ] APDV-030 [owner=codex] [deps=APDV-020] [scope=apps/android/scripts/Smoke-Emulator.ps1,apps/android/README.md,docs/workstreams/android-playback-depth-validation]
  Goal: After leaving the player, read back server **User Playback State** and
  prove the player exit report reached the server.
  Validation: focused `profile-with-media` smoke command; generated readback
  artifact contains updated server state; `git diff --check`.
  Review: evidence must be token-safe and must not rely on Android
  device-local resume.
  Evidence: `EVIDENCE_AND_GATES.md`, generated server readback artifact.
  Handoff: APDV-040 can close or split HLS/remux/session follow-ons.

## M3 - Closeout

- [ ] APDV-040 [owner=planner] [deps=APDV-030] [scope=docs/workstreams/android-playback-depth-validation]
  Goal: Verify playback depth evidence, update docs, and close or split deeper
  playback follow-ons.
  Validation: fresh focused smoke evidence and `git diff --check`.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Handoff: HLS/remux/session cancellation, longer watched-threshold media, and
  playback quality checks remain separate lanes.
