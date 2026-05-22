# Android Local Resume Smoke Evidence - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Boundary Freeze

- [x] ALR-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-local-resume-smoke-evidence]
  Goal: Open the lane and freeze the distinction between device-local resume
  smoke evidence and server-authoritative **User Playback State**.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/android-local-resume-smoke-evidence/DESIGN.md`
  Handoff: Completed on 2026-05-19. First implementation slice is debug-only
  fixture seeding plus focused `profile-with-media` smoke assertions.

## M1 - Device-Local Resume Smoke Slice

- [x] ALR-020 [owner=codex] [deps=ALR-010] [scope=apps/android/app/src/debug,apps/android/app/src/testDebug,apps/android/scripts]
  Goal: Allow the debug smoke fixture to inject a device-local playback
  position and make `profile-with-media` prove the local resume UI path.
  Validation:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.smoke.DebugSmokeFixtureSeedActivityTest --no-daemon`
  plus
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`
  and `git diff --check`.
  Review: Confirm the new seed path is debug-only, uses device-local storage,
  and does not introduce server-authoritative **User Playback State** wording.
  Evidence: `EVIDENCE_AND_GATES.md`.
  Handoff: DONE on 2026-05-19. Extended the debug seed request/provider to
  persist optional device-local resume state, resolved the fixture Media Item
  and Media Source ids from the running server, and asserted local resume copy,
  `Start resume`, player `Local resume 0:01`, and forbidden cross-device state
  wording in smoke criteria. Latest evidence:
  `apps/android/build/smoke/20260519-102517-profile-with-media-emulator-5554/report.md`.

## M2 - Closeout

- [x] ALR-030 [owner=planner] [deps=ALR-020] [scope=docs/workstreams/android-local-resume-smoke-evidence]
  Goal: Verify evidence, close this lane, and keep deeper playback/golden/CI
  work split.
  Validation: fresh ALR-020 gates and closeout doc updates.
  Review: Use review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: DONE on 2026-05-19. Lane is closed. CI/device-farm execution,
  golden screenshot diffing, and deeper playback duration/seek validation stay
  as follow-ons.
