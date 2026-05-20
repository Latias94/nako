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

## JUnit Contract

File name:

- `report.junit.xml` next to each command's existing `report.md` and
  `report.json`.

XML shape:

- Root element: `<testsuites>`.
- One top-level `<testsuite>` per validation command invocation.
- A suite may include `<properties>` with path metadata.
- A testcase may include `<failure>` or `<skipped>`, but should not include
  screenshots, full UI hierarchy XML, bearer tokens, or source locators.

Smoke regression suite:

- Suite name: `taru.android.smoke-regression`.
- Suite properties:
  - `report.markdown`
  - `report.json`
  - `fixture_server_port`
  - `requested_serial`
  - `started_at`
  - `finished_at`
- Testcase classname: `taru.android.smoke`.
- Testcase name: `state.<state-name>`, for example
  `state.profile-with-media`.
- The wrapper build step is represented as `step.android-build` in the same
  suite so CI can annotate build failures or intentional `SkipBuild` runs.
- Passing smoke state: testcase has no child result element.
- Failed smoke state: testcase has `<failure type="<category>">` with a
  token-safe message and report/evidence paths.
- Skipped or not-run state: testcase has `<skipped>` with a token-safe reason.

Local validation suite:

- Suite name: `taru.android.local-validation`.
- Suite properties:
  - `report.markdown`
  - `report.json`
  - `started_at`
  - `finished_at`
  - `smoke.report.markdown` when present
  - `smoke.report.json` when present
  - `smoke.report.junit` when present
- Testcase classname: `taru.android.validation`.
- Testcase names:
  - `step.android-build`
  - `step.android-unit-tests`
  - `step.smoke-regression`
- Passing validation step: testcase has no child result element.
- Failed validation step: testcase has `<failure type="<step-status>">` with a
  token-safe message and log/report paths.
- Intentionally skipped smoke: `step.smoke-regression` has `<skipped>` with
  `SkipSmoke` as the reason.

Escaping and safety:

- Use XML APIs or explicit XML escaping for attribute values and text nodes.
- Paths are allowed because they already appear in Markdown and JSON reports.
- Never include raw command output inline. Link log/report paths instead.
- Preserve existing Markdown and JSON outputs byte-for-byte where practical;
  JUnit generation should be additive.

## Closeout Condition

This lane can close when both Android validation scripts emit parseable JUnit
XML, the XML is documented and validated by fresh commands, and remaining CI
upload or golden screenshot work is explicitly left to separate lanes.
