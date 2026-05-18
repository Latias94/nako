# Android Client QA Harness — TODO

Status: Active
Last updated: 2026-05-18

## M0 — Scope And Evidence Freeze

- [x] ACQ-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-client-qa-harness]
  Goal: Open the Android QA harness lane and freeze the testing boundary.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/android-client-qa-harness/DESIGN.md`
  Handoff: Completed before implementation starts.

## M1 — Local Smoke Command

- [ ] ACQ-020 [owner=codex] [deps=ACQ-010] [scope=apps/android/README.md,apps/android/scripts]
  Goal: Add a documented local smoke command for build, install, launch, and
  basic evidence capture against an already running emulator.
  Validation: Run the smoke command against the available emulator, plus
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
  and `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`.
  Review: Use review-workstream before accepting completion.
  Evidence: `EVIDENCE_AND_GATES.md`, generated output path summary.
  Handoff: Script must not require committing screenshots.

## M2 — Fixture And State Strategy

- [ ] ACQ-030 [owner=codex] [deps=ACQ-020] [scope=apps/android,docs/workstreams/android-client-qa-harness]
  Goal: Define the repeatable fixture/state strategy for setup, Home,
  Settings, Server Profile, empty/error states, and future detail/player checks.
  Validation: JVM tests or documented manual smoke evidence prove token-safe
  and locator-safe fixture behavior.
  Review: Use review-workstream before accepting completion.
  Evidence: `EVIDENCE_AND_GATES.md`, fixture/state documentation.
  Handoff: Split server-backed demo fixture work if Android-only state is not
  enough.

## M3 — Emulator Surface Coverage

- [ ] ACQ-040 [owner=codex] [deps=ACQ-030] [scope=apps/android/scripts,apps/android/app/src]
  Goal: Cover the first repeatable emulator surfaces: setup or existing
  profile launch, Home, Settings, and Server Profile screenshots with explicit
  pass/fail criteria.
  Validation: Smoke command produces named screenshots and exits non-zero on
  install/launch/capture failure.
  Review: Use review-workstream before accepting completion.
  Evidence: `EVIDENCE_AND_GATES.md`, screenshot path summary.
  Handoff: Detail/player screenshot coverage can split if it needs server
  fixture work.

## M4 — Closeout

- [ ] ACQ-050 [owner=planner] [deps=ACQ-040] [scope=docs/workstreams/android-client-qa-harness]
  Goal: Verify the harness, update evidence, and close or split CI/golden
  follow-ons.
  Validation: Android unit tests, debug assemble, smoke command on emulator,
  and `git diff --check`.
  Review: Use review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Handoff: Record manual-only gaps and CI/golden screenshot follow-ons.
