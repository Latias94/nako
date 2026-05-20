# Android JUnit Validation Reports - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

This lane follows the closed `android-structured-validation-reports` workstream.
Markdown and JSON reports already exist for smoke regression and local Android
validation. AJVR-010 is complete: the JUnit XML contract is frozen in
`DESIGN.md`. AJVR-020 is complete: `Smoke-Regression.ps1` now writes
`report.junit.xml` next to `report.md` and `report.json`. AJVR-030 is
complete: `Validate-AndroidLocal.ps1` now writes local validation
`report.junit.xml` and links delegated smoke JUnit paths.

## Active Task

- Task ID: AJVR-040
- Owner: Codex
- Files:
  - `docs/workstreams/android-junit-validation-reports/`
- Validation:
  - `pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Android-JUnitReport.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Regression.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Validate-AndroidLocal.ps1' -Raw)) | Out-Null"`
  - `pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup -SkipBuild -RetriesPerState 0`
  - `pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke`
  - parse generated JUnit XML reports
  - `git diff --check`
- Status: READY

## Decisions

- Reuse the existing PowerShell validation scripts.
- Treat JUnit XML as a report adapter over existing status/path data.
- Do not change smoke fixture behavior or duplicate smoke navigation.
- Do not include secrets, source locators, screenshot bytes, or hierarchy XML
  in JUnit output.
- CI upload/artifact retention and golden visual diffing stay out of scope.
- `report.junit.xml` is the committed contract name for generated JUnit XML.
- Smoke suite name is `taru.android.smoke-regression`.
- Local validation suite name is `taru.android.local-validation`.
- Smoke JUnit includes `step.android-build` and `state.<state-name>`
  testcases.
- Local validation JUnit includes `step.android-unit-tests`,
  `step.android-build`, and `step.smoke-regression` testcases.
- Shared JUnit XML helper lives in `apps/android/scripts/Android-JUnitReport.ps1`.

## Blockers

- None for AJVR-040.

## Next Recommended Action

- Execute AJVR-040: verify both generated JUnit report paths, update closeout
  docs, and close the lane. CI upload/artifact retention and golden visual
  diffing remain separate follow-ons.
