# Android Structured Validation Reports - Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

ASVR-010, ASVR-020, ASVR-030, and ASVR-040 are complete. The lane is closed.
Both Android validation scripts now write `report.json` next to `report.md`.

## Closeout Evidence

- `apps/android/build/smoke-regression-asvr/20260519-171154/report.json`
- `apps/android/build/validation-asvr/20260519-170805/report.json`
- `git diff --check`

## Follow-Ons

Split these into new lanes if needed:

- JUnit XML export for CI test reporting.
- CI upload/artifact retention policy.
- Golden visual diffing over smoke screenshots.

## Constraints

- Do not rewrite PowerShell in Python in this lane.
- Do not duplicate smoke UI navigation in validation scripts.
- Do not include bearer tokens, source locators, or screenshot binary content
  in JSON reports.
- Generated reports remain under `apps/android/build/`.
