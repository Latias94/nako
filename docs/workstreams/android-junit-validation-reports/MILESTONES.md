# Android JUnit Validation Reports - Milestones

Status: Closed
Last updated: 2026-05-21

## M1 - Contract

Status: Complete

Exit criteria:

- The JUnit XML contract is documented before implementation.
- Suite and testcase naming are stable enough for CI history.
- Token-safety constraints are explicit.

Evidence:

- `DESIGN.md` defines `report.junit.xml`, `<testsuites>` root shape, suite
  names, testcase names, pass/fail/skipped mapping, allowed properties, and
  token-safety constraints.

## M2 - Smoke Regression XML

Status: Complete

Exit criteria:

- `Smoke-Regression.ps1` emits `report.junit.xml`.
- Each requested smoke state appears as one testcase.
- Failure category, attempt count, and report paths are available without
  embedding screenshots, hierarchy XML, bearer tokens, or source locators.

Evidence:

- `Smoke-Regression.ps1` writes `report.junit.xml` and prints the JUnit report
  path.
- Smoke JSON now includes `report_junit`.
- Focused `empty-setup` smoke generated parseable JUnit XML with
  `step.android-build` and `state.empty-setup` testcases.

## M3 - Local Validation XML

Status: Complete

Exit criteria:

- `Validate-AndroidLocal.ps1` emits `report.junit.xml`.
- Build/unit/smoke validation steps appear as stable testcases.
- Delegated smoke Markdown/JSON/JUnit paths are linked as text metadata rather
  than copied into local validation XML.

Evidence:

- `Validate-AndroidLocal.ps1` writes `report.junit.xml` and prints the JUnit
  report path.
- Local validation JSON includes `report_junit` and delegated
  `smoke_junit`.
- Focused `-SkipSmoke` validation generated parseable JUnit XML with
  `step.android-unit-tests`, `step.android-build`, and
  `step.smoke-regression` testcases.

## M4 - Closeout

Status: Complete

Exit criteria:

- [x] Fresh parseable JUnit evidence is recorded.
- [x] Existing Markdown and JSON outputs remain compatible.
- [x] CI upload/artifact retention and golden visual diffing remain separate
  follow-ons.
