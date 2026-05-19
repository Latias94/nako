# Android Client QA Harness — TODO

Status: Completed
Last updated: 2026-05-18

## M0 — Scope And Evidence Freeze

- [x] ACQ-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-client-qa-harness]
  Goal: Open the Android QA harness lane and freeze the testing boundary.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/android-client-qa-harness/DESIGN.md`
  Handoff: Completed before implementation starts.

## M1 — Local Smoke Command

- [x] ACQ-020 [owner=codex] [deps=ACQ-010] [scope=apps/android/README.md,apps/android/scripts]
  Goal: Add a documented local smoke command for build, install, launch, and
  basic evidence capture against an already running emulator.
  Validation: Run the smoke command against the available emulator, plus
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
  and `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`.
  Review: Use review-workstream before accepting completion.
  Evidence: `EVIDENCE_AND_GATES.md`, generated output path summary.
  Handoff: Completed on 2026-05-18. Script and README implementation are
  local. Android unit tests, debug assemble, and emulator smoke validation all
  passed. Evidence path:
  `apps/android/build/smoke/20260518-204538-emulator-5554/`.

## M2 — Fixture And State Strategy

- [x] ACQ-030 [owner=codex] [deps=ACQ-020] [scope=apps/android,docs/workstreams/android-client-qa-harness]
  Goal: Define the repeatable fixture/state strategy for setup, Home,
  Settings, Server Profile, empty/error states, and future detail/player checks.
  Validation: JVM tests or documented manual smoke evidence prove token-safe
  and locator-safe fixture behavior.
  Review: Use review-workstream before accepting completion.
  Evidence: `EVIDENCE_AND_GATES.md`, fixture/state documentation.
  Handoff: Completed on 2026-05-18. `current-state` and `empty-setup` modes
  are documented in `apps/android/SMOKE_FIXTURES.md`; smoke supports
  `-ResetAppData`; empty setup evidence was generated under
  `apps/android/build/smoke/20260518-205953-empty-setup-emulator-5554/`.

## M3 — Emulator Surface Coverage

- [x] ACQ-040 [owner=codex] [deps=ACQ-030] [scope=apps/android/scripts,apps/android/app/src]
  Goal: Cover the first repeatable emulator surfaces: setup or existing
  profile launch, Home, Settings, and Server Profile screenshots with explicit
  pass/fail criteria.
  Validation: Smoke command produces named screenshots and exits non-zero on
  install/launch/capture failure.
  Review: Use review-workstream before accepting completion.
  Evidence: `EVIDENCE_AND_GATES.md`, screenshot path summary.
  Handoff: Completed on 2026-05-18. `empty-setup` captures setup evidence, and
  `profile-missing-token` captures Home, Settings, and Server Profile shell
  evidence with named screenshots, UI hierarchy dumps, and criteria files.
  Latest evidence:
  `apps/android/build/smoke/20260518-214452-empty-setup-emulator-5554/` and
  `apps/android/build/smoke/20260518-214533-profile-missing-token-emulator-5554/`.

## M4 — Closeout

- [x] ACQ-050 [owner=planner] [deps=ACQ-040] [scope=docs/workstreams/android-client-qa-harness]
  Goal: Verify the harness, update evidence, and close or split CI/golden
  follow-ons.
  Validation: Android unit tests, debug assemble, smoke command on emulator,
  and `git diff --check`.
  Review: Use review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Handoff: Completed on 2026-05-18. Fresh Android unit tests, debug assemble,
  `empty-setup` smoke, `profile-missing-token` smoke, and diff hygiene passed.
  The lane is closed with CI, golden visual diffing, server-backed demo data,
  instrumentation, and detail/player smoke coverage recorded as follow-ons.
