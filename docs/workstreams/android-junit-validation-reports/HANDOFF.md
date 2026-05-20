# Android JUnit Validation Reports - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

This lane follows the closed `android-structured-validation-reports` workstream.
Markdown and JSON reports already exist for smoke regression and local Android
validation. AJVR-010 is complete: the JUnit XML contract is frozen in
`DESIGN.md`. AJVR-020 is complete: `Smoke-Regression.ps1` now writes
`report.junit.xml` next to `report.md` and `report.json`.

## Active Task

- Task ID: AJVR-030
- Owner: Codex
- Files:
  - `apps/android/scripts/Validate-AndroidLocal.ps1`
  - `apps/android/README.md`
  - `docs/workstreams/android-junit-validation-reports/`
- Validation:
  - `pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Validate-AndroidLocal.ps1' -Raw)) | Out-Null"`
  - `pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke`
  - parse generated `report.junit.xml`
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

## Blockers

- None for AJVR-030.

## Next Recommended Action

- Execute AJVR-030: add `Validate-AndroidLocal.ps1` JUnit XML output and link
  delegated smoke JUnit paths instead of duplicating smoke state detail.
