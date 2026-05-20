# Android JUnit Validation Reports

Status: Active
Last updated: 2026-05-20

## Why This Lane Exists

`docs/workstreams/android-structured-validation-reports/` closed after adding
token-safe JSON output to `Smoke-Regression.ps1` and
`Validate-AndroidLocal.ps1`. That gives machines stable data, but most CI
systems still expect JUnit XML for test reporting, failure annotations, and
historical test surfaces.

## Problem

Android local validation can prove build, unit, and smoke gates, but CI cannot
consume those results as normal test suites without custom parsing. Developers
need the existing validation commands to emit additive JUnit XML reports
without changing smoke navigation, fixture behavior, or the existing Markdown
and JSON report contracts.

## Target State

- `Smoke-Regression.ps1` writes a token-safe JUnit XML report next to
  `report.md` and `report.json`.
- `Validate-AndroidLocal.ps1` writes a token-safe JUnit XML report next to
  `report.md` and `report.json`.
- XML results model validation steps and smoke states as stable test cases.
- Existing command behavior and Markdown/JSON outputs remain compatible.
- Generated reports stay under `apps/android/build/` and are not committed.

## In Scope

- Additive JUnit XML report generation for smoke regression runs.
- Additive JUnit XML report generation for local validation runs.
- Script parse checks, focused generated XML parse checks, and docs updates.
- Token-safe failure messages and report paths.

## Out Of Scope

- CI workflow files, upload/artifact retention policy, or device-farm setup.
- Golden screenshot diffing.
- Rewriting validation scripts in Python.
- Changing smoke fixture state, navigation, screenshots, or hierarchy capture.
- Duplicating smoke state detail inside `Validate-AndroidLocal.ps1`; the local
  validation JUnit should link delegated smoke reports instead.

## Architecture Direction

Treat JUnit XML as an adapter over the existing structured report data. The
scripts should continue to own orchestration and should emit XML from the same
status/category/path values already used by Markdown and JSON reports. The XML
must be conservative: stable suite names, stable case names, escaped text, no
secrets, and no embedded screenshots or UI hierarchy payloads.

## Closeout Condition

This lane can close when both Android validation scripts emit parseable JUnit
XML, the XML is documented and validated by fresh commands, and remaining CI
upload or golden screenshot work is explicitly left to separate lanes.
