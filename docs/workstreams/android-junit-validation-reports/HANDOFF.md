# Android JUnit Validation Reports - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

This lane follows the closed `android-structured-validation-reports` workstream.
Markdown and JSON reports already exist for smoke regression and local Android
validation. AJVR-010 is complete: the JUnit XML contract is frozen in
`DESIGN.md`.

## Active Task

- Task ID: AJVR-020
- Owner: Codex
- Files:
  - `apps/android/scripts/Smoke-Regression.ps1`
  - `apps/android/SMOKE_FIXTURES.md`
  - `docs/workstreams/android-junit-validation-reports/`
- Validation:
  - `pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Regression.ps1' -Raw)) | Out-Null"`
  - `pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup -SkipBuild -RetriesPerState 0`
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

## Blockers

- None for AJVR-020.

## Next Recommended Action

- Execute AJVR-020: add `Smoke-Regression.ps1` JUnit XML output first, without
  changing smoke navigation or Markdown/JSON behavior.
