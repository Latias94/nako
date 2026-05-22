# Android JUnit Validation Reports - Handoff

Status: Closed
Last updated: 2026-05-21

## Current State

This lane is closed. It follows the closed
`android-structured-validation-reports` workstream.
Markdown and JSON reports already exist for smoke regression and local Android
validation. AJVR-010 is complete: the JUnit XML contract is frozen in
`DESIGN.md`. AJVR-020 is complete: `Smoke-Regression.ps1` now writes
`report.junit.xml` next to `report.md` and `report.json`. AJVR-030 is
complete: `Validate-AndroidLocal.ps1` now writes local validation
`report.junit.xml` and links delegated smoke JUnit paths.

## Closeout

AJVR-040 completed on 2026-05-21. Fresh script parse, focused smoke regression
JUnit generation, focused local validation JUnit generation, generated XML
parse checks, focused connection UI unit tests, and `git diff --check` passed.

## Decisions

- Reuse the existing PowerShell validation scripts.
- Treat JUnit XML as a report adapter over existing status/path data.
- Do not change smoke fixture behavior or duplicate smoke navigation.
- Do not include secrets, source locators, screenshot bytes, or hierarchy XML
  in JUnit output.
- CI upload/artifact retention and golden visual diffing stay out of scope.
- `report.junit.xml` is the committed contract name for generated JUnit XML.
- Smoke suite name is `nako.android.smoke-regression`.
- Local validation suite name is `nako.android.local-validation`.
- Smoke JUnit includes `step.android-build` and `state.<state-name>`
  testcases.
- Local validation JUnit includes `step.android-unit-tests`,
  `step.android-build`, and `step.smoke-regression` testcases.
- Shared JUnit XML helper lives in `apps/android/scripts/Android-JUnitReport.ps1`.

## Blockers

- None for this lane.

## Next Recommended Action

- Open separate lanes for CI upload/artifact retention or golden visual
  diffing if those become release requirements.
