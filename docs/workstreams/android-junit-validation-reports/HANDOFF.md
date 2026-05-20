# Android JUnit Validation Reports - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

This lane follows the closed `android-structured-validation-reports` workstream.
Markdown and JSON reports already exist for smoke regression and local Android
validation. The remaining report-format gap is additive JUnit XML for CI test
surfaces.

## Active Task

- Task ID: AJVR-010
- Owner: Codex
- Files:
  - `docs/workstreams/android-junit-validation-reports/`
- Validation:
  - workstream docs are internally consistent.
  - `Get-Content -LiteralPath 'docs/workstreams/android-junit-validation-reports/WORKSTREAM.json' -Raw | ConvertFrom-Json | Out-Null`
- Status: READY

## Decisions

- Reuse the existing PowerShell validation scripts.
- Treat JUnit XML as a report adapter over existing status/path data.
- Do not change smoke fixture behavior or duplicate smoke navigation.
- Do not include secrets, source locators, screenshot bytes, or hierarchy XML
  in JUnit output.
- CI upload/artifact retention and golden visual diffing stay out of scope.

## Blockers

- None for AJVR-010.

## Next Recommended Action

- Execute AJVR-010, then set a focused goal for AJVR-020 to add
  `Smoke-Regression.ps1` JUnit XML output first.
