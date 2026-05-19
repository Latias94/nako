# Android Developer Validation Entrypoint - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Scope And Command Contract

- [x] ADV-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-developer-validation-entrypoint]
  Goal: Open the lane and freeze the command contract: one local validation
  entrypoint that composes existing Android gates without taking ownership of
  smoke navigation.
  Validation: Workstream docs exist and agree.
  Review: Use review-workstream before closeout.
  Evidence: `docs/workstreams/android-developer-validation-entrypoint/DESIGN.md`
  Handoff: Completed on 2026-05-19. First implementation slice is the local
  PowerShell entrypoint and documentation.

## M1 - Local Validation Entrypoint

- [x] ADV-020 [owner=codex] [deps=ADV-010] [scope=apps/android/scripts,apps/android/README.md,docs/workstreams/android-developer-validation-entrypoint]
  Goal: Add `Validate-AndroidLocal.ps1`, a developer-facing command that runs
  Android JVM tests, optional debug build, and optional smoke regression while
  writing a combined report.
  Validation:
  `pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke`
  plus `git diff --check`; run the default command if an emulator is available.
  Review: Check that it delegates smoke to `Smoke-Regression.ps1` and does not
  duplicate UI/ADB navigation.
  Evidence: `EVIDENCE_AND_GATES.md`, generated validation report path.
  Handoff: DONE on 2026-05-19. Added `Validate-AndroidLocal.ps1`, documented it
  in the Android README, validated both `-SkipSmoke` and default modes, and
  confirmed the default mode links to the delegated smoke regression report.

## M2 - Closeout

- [x] ADV-030 [owner=planner] [deps=ADV-020] [scope=docs/workstreams/android-developer-validation-entrypoint]
  Goal: Verify the entrypoint, update evidence, and close or split CI/report
  format follow-ons.
  Validation: fresh validation command evidence and `git diff --check`.
  Review: Use review-workstream and verify-rust-workstream before closeout.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: DONE on 2026-05-19. CI/device-farm packaging, golden visual diffs,
  and structured JSON/JUnit export remain deferred follow-ons.
