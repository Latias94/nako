# Android Server-Backed Demo Fixtures — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Scope And Evidence Freeze

- [x] ASD-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-server-backed-demo-fixtures]
  Goal: Open the Android server-backed demo fixtures lane and freeze the Public
  Client API fixture boundary.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: `docs/workstreams/android-server-backed-demo-fixtures/DESIGN.md`
  Handoff: Completed on 2026-05-18. Android must not fake server-backed media
  fixture data; media smoke evidence must come through Public Client API route
  shapes or an explicit public-route-compatible test-server harness.

## M1 — Fixture Contract Discovery

- [x] ASD-020 [owner=codex] [deps=ASD-010] [scope=crates/taru-api,crates/taru-client,apps/android,docs/workstreams/android-server-backed-demo-fixtures]
  Goal: Inventory the minimal Public Client API responses needed for the first
  Android media smoke state and identify whether a seeded Taru server or local
  test-server harness is the cleanest first implementation.
  Validation: Focused API/Android client tests or a documented route matrix
  prove every required screen has a public route source.
  Review: Use review-workstream before accepting completion.
  Evidence: `EVIDENCE_AND_GATES.md`, route matrix in this workstream or linked
  docs.
  Handoff: DONE on 2026-05-18. `ROUTE_MATRIX.md` records that the first
  implementation should use a real seeded local `taru-server` reached by
  Android through `adb reverse`; a public-route-compatible test server remains
  only a fallback if seeded startup proves too brittle.

## M2 — Server-Backed Fixture Provider

- [x] ASD-030 [owner=codex] [deps=ASD-020] [scope=apps/android/scripts,apps/android/app/src,apps/android/app/src/test,apps/android/README.md,apps/android/SMOKE_FIXTURES.md,docs/workstreams/android-server-backed-demo-fixtures]
  Goal: Provide the first deterministic fixture endpoint or startup path that
  returns safe demo Media Libraries, Media Items, Item Detail, Media Sources,
  and playback decision data through Public Client API route shapes.
  Validation: A local command starts or selects the fixture provider, and
  request-level tests prove responses are token-safe and locator-safe.
  Review: Use review-workstream for API boundary and safety review.
  Evidence: `EVIDENCE_AND_GATES.md`, startup command, focused server/API test
  output.
  Handoff: DONE on 2026-05-18. Added
  `apps/android/scripts/Start-DemoFixtureServer.ps1`, aligned Android
  `ClientTranscodePlan` with the Public Client API by removing the internal
  `input_locator` requirement, and validated seeded `Night Harbor` Public
  Client API responses against a short-lived local server.

## M3 — Android Media Smoke State

- [x] ASD-040 [owner=codex] [deps=ASD-030] [scope=apps/android/scripts,apps/android/app/src,apps/android/SMOKE_FIXTURES.md,apps/android/README.md]
  Goal: Add a named Android smoke fixture state that seeds a safe Server
  Profile, consumes the fixture provider, navigates Home -> detail -> source
  picker -> player-safe launch, and captures named evidence.
  Validation: `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`,
  `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`,
  and `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`.
  Review: Use review-workstream before accepting completion.
  Evidence: `EVIDENCE_AND_GATES.md`, generated smoke output path summary.
  Handoff: DONE on 2026-05-18. `Smoke-Emulator.ps1 -FixtureState
  profile-with-media` prepares the real server-backed `Night Harbor` fixture,
  seeds a debug-only Server Profile and encrypted token value, captures Home,
  detail, source picker, and player evidence, and keeps screenshots/reports as
  generated local artifacts. Full playback quality, CI, golden screenshot
  diffing, and HLS/remux/session depth remain follow-ons.

## M4 — Safety, Verification, And Closeout

- [x] ASD-050 [owner=planner] [deps=ASD-040] [scope=docs/workstreams/android-server-backed-demo-fixtures,apps/android]
  Goal: Verify the fixture lane, close it, or split CI/golden/deeper playback
  follow-ons.
  Validation: Fresh Android unit tests, debug assemble, server/fixture tests,
  media smoke run, and `git diff --check`.
  Review: Use review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Handoff: DONE on 2026-05-18. Fresh Android unit, debug assemble,
  server-backed fixture prepare, media smoke, and `git diff --check` gates
  passed. The lane is closed with CI/golden visual diffing, HLS/remux/session
  depth, and longer playback quality validation deferred as follow-ons.
