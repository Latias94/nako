# Android End-To-End Validation Hardening

Status: Active
Last updated: 2026-05-20

## Problem

Android now has local unit/build validation, state-level smoke navigation, and
smoke regression wrappers. The deep user path is covered, but evidence is still
hard to consume programmatically at the fixture-state boundary:

- `Smoke-Emulator.ps1` writes state screenshots, UI dumps, criteria files, and a
  Markdown summary, but no state-level structured report.
- `Smoke-Regression.ps1` records evidence directories and logs, but not stable
  links to each state's own Markdown and JSON reports.
- `Validate-AndroidLocal.ps1` links the delegated regression report only at the
  top level, so triage still requires manual drilling into generated folders.

This makes the current end-to-end gate useful to a human, but weaker for future
CI, dashboards, review bots, and precise failure handoff.

## Target State

- Each smoke fixture state writes a token-safe `report.json` next to
  `report.md`.
- The regression report links each state's evidence directory, Markdown report,
  JSON report, log, status, attempts, category, and rerun command.
- The local validation report continues to delegate to smoke regression, but can
  point to structured downstream evidence without duplicating smoke internals.
- The existing navigation and surface assertions remain unchanged.

## Scope

- `apps/android/scripts/Smoke-Emulator.ps1`
- `apps/android/scripts/Smoke-Regression.ps1`
- `apps/android/scripts/Validate-AndroidLocal.ps1` if downstream evidence links
  need to be surfaced at the top validation layer.
- Workstream docs under this directory.

## Non-Goals

- Do not rewrite the harness in Python.
- Do not introduce screenshot golden diffing.
- Do not add CI/device-farm packaging.
- Do not change UI copy or Android app runtime behavior.
- Do not commit generated smoke reports, screenshots, dumps, or fixture data.

## Architecture Direction

Keep ADB/UI navigation in `Smoke-Emulator.ps1`, multi-state orchestration in
`Smoke-Regression.ps1`, and developer-facing command composition in
`Validate-AndroidLocal.ps1`. Harden their contracts with small structured JSON
documents so each layer can link evidence from the layer below instead of
duplicating its details.
