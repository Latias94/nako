# Android JUnit Validation Reports - TODO

Status: Active
Last updated: 2026-05-20

## Task Ledger

- [x] AJVR-010 - Freeze JUnit report contract.
  - Owner: Codex
  - Dependencies: closed `docs/workstreams/android-structured-validation-reports/`.
  - Scope:
    - `docs/workstreams/android-junit-validation-reports/`
    - existing validation report schemas.
  - Validation:
    - Workstream docs define suite names, testcase names, pass/fail mapping,
      and token-safety constraints.
  - Evidence: planning docs.
    Completed on 2026-05-20:
    `DESIGN.md` now freezes file name, root XML shape, suite names, testcase
    names, pass/fail/skipped mapping, allowed properties, and token-safety
    constraints for smoke regression and local validation JUnit XML reports.

- [x] AJVR-020 - Add smoke regression JUnit report.
  - Owner: Codex
  - Dependencies: AJVR-010.
  - Scope:
    - `apps/android/scripts/Smoke-Regression.ps1`
    - `apps/android/SMOKE_FIXTURES.md`
    - focused smoke report validation.
  - Validation:
    - script parse check passes.
    - a focused smoke regression run writes `report.junit.xml`.
    - generated XML parses and records one testcase per requested smoke state.
    - existing Markdown and JSON reports remain present.
  - Evidence: generated smoke regression report path.
    Completed on 2026-05-20:
    `Smoke-Regression.ps1` now writes `report.junit.xml` next to `report.md`
    and `report.json`, prints the JUnit report path, and records
    `report_junit` in the JSON report. Focused `empty-setup` smoke generated a
    parseable JUnit report with `step.android-build` and `state.empty-setup`
    testcases while preserving Markdown and JSON outputs.

- [x] AJVR-030 - Add local validation JUnit report.
  - Owner: Codex
  - Dependencies: AJVR-020.
  - Scope:
    - `apps/android/scripts/Android-JUnitReport.ps1`
    - `apps/android/scripts/Smoke-Regression.ps1`
    - `apps/android/scripts/Validate-AndroidLocal.ps1`
    - `apps/android/README.md`
    - focused local validation report validation.
  - Validation:
    - script parse check passes.
    - `Validate-AndroidLocal.ps1 -SkipSmoke` writes `report.junit.xml`.
    - generated XML parses and records build/unit/smoke step testcases.
    - delegated smoke reports are linked, not duplicated.
  - Evidence: generated local validation report path.
    Completed on 2026-05-20:
    `Validate-AndroidLocal.ps1` now writes `report.junit.xml`, prints the
    JUnit report path, records `report_junit` in `report.json`, and links
    delegated smoke Markdown/JSON/JUnit paths when smoke runs. Focused
    `-SkipSmoke` validation generated a parseable local validation JUnit report
    with `step.android-unit-tests`, `step.android-build`, and
    `step.smoke-regression` testcases.

- [ ] AJVR-040 - Verify and close.
  - Owner: Codex
  - Dependencies: AJVR-030.
  - Scope:
    - `docs/workstreams/android-junit-validation-reports/`
  - Validation:
    - fresh generated JUnit evidence is recorded.
    - `git diff --check` passes.
    - remaining CI upload and golden screenshot work is split or deferred.
  - Evidence: closeout docs.
