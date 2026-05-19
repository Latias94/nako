# Android Structured Validation Reports - TODO

Status: Closed
Last updated: 2026-05-19

## M0 - Scope And Report Contract

- [x] ASVR-010 [owner=planner] [deps=none] [scope=docs/workstreams/android-structured-validation-reports]
  Goal: Open the lane and freeze the first JSON report contract for Android
  smoke regression and local validation.
  Validation: workstream docs exist and agree.
  Review: confirm this does not reopen CI/golden screenshot scope.
  Evidence: `DESIGN.md`, `TODO.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`.
  Handoff: First implementation slice is ASVR-020.

## M1 - Smoke Regression JSON Report

- [x] ASVR-020 [owner=codex] [deps=ASVR-010] [scope=apps/android/scripts/Smoke-Regression.ps1,apps/android/SMOKE_FIXTURES.md,docs/workstreams/android-structured-validation-reports]
  Goal: Add `report.json` output to `Smoke-Regression.ps1` without changing
  state execution or Markdown behavior.
  Validation: script parse check; a focused smoke regression command that
  generates parseable `report.json`; `git diff --check`.
  Review: JSON must be token-safe and derive from existing result objects.
  Evidence: `EVIDENCE_AND_GATES.md`, generated smoke regression report path.
  Handoff: DONE on 2026-05-19. `Smoke-Regression.ps1` writes `report.json`
  next to `report.md`, prints its path, and preserves existing Markdown
  behavior.

## M2 - Local Validation JSON Report

- [x] ASVR-030 [owner=codex] [deps=ASVR-020] [scope=apps/android/scripts/Validate-AndroidLocal.ps1,apps/android/README.md,docs/workstreams/android-structured-validation-reports]
  Goal: Add `report.json` output to `Validate-AndroidLocal.ps1`, including a
  pointer to delegated smoke regression JSON when smoke runs.
  Validation: `pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke`; parse generated JSON; `git diff --check`.
  Review: Do not duplicate smoke state detail beyond linking delegated report
  paths.
  Evidence: `EVIDENCE_AND_GATES.md`, generated validation report path.
  Handoff: DONE on 2026-05-19. `Validate-AndroidLocal.ps1` writes
  `report.json`, links delegated smoke Markdown/JSON reports when smoke runs,
  and normalizes relative `-OutputRoot` to avoid Gradle working-directory path
  drift.

## M3 - Closeout

- [x] ASVR-040 [owner=planner] [deps=ASVR-030] [scope=docs/workstreams/android-structured-validation-reports]
  Goal: Verify structured reports, update evidence, and close or split
  remaining report-format follow-ons.
  Validation: fresh parseable report evidence and `git diff --check`.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`.
  Handoff: DONE on 2026-05-19. JSON report evidence is captured and the lane is
  closed. JUnit XML, CI upload, and golden visual diffing remain separate
  follow-ons.
