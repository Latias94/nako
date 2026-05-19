# Android Structured Validation Reports

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

Android now has reliable local smoke regression and a developer-facing local
validation entrypoint, but both primarily write Markdown reports. That is
enough for manual handoff, but it keeps CI, trend analysis, and future visual
evidence tooling dependent on parsing presentation text.

This lane adds stable machine-readable report files while keeping the existing
PowerShell orchestration thin and preserving Markdown for humans.

## Target State

When this lane closes:

- `Smoke-Regression.ps1` writes `report.json` next to `report.md`.
- `Validate-AndroidLocal.ps1` writes `report.json` next to `report.md`.
- JSON fields are stable, token-safe, path-oriented, and suitable for CI or
  follow-on JUnit/golden visual tooling.
- Existing Markdown report content and command behavior remain compatible.
- Generated reports stay under `apps/android/build/` and are not committed.

## In Scope

- Structured JSON report shape for smoke regression runs.
- Structured JSON report shape for local Android validation runs.
- Documentation and workstream evidence for the new report artifacts.
- Parse-level validation of generated JSON.

## Out Of Scope

- JUnit XML export.
- CI/device-farm packaging.
- Golden screenshot comparison.
- Rewriting PowerShell scripts in Python.
- Changing ADB/UI navigation, fixture setup, or smoke criteria semantics.

## Architecture Direction

Treat structured reports as an additional adapter at the existing validation
report seam. The scripts should continue to own orchestration and delegate
state-specific UI work to the existing smoke commands. JSON output should be
derived from the same in-memory result objects that produce Markdown so the two
formats cannot drift in behavior.

The first report schema is intentionally local and versioned by a
`schema_version` field. It should include timestamps, requested options,
overall result, step/state results, log/evidence paths, errors, and delegated
report paths. It must not include bearer tokens, raw source locators, or
screenshot binary content.

This lane is closed. Both Android validation scripts now produce additive JSON
reports next to their existing Markdown reports.
