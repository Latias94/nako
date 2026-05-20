# Android End-To-End Validation Hardening - Handoff

Status: Active
Last updated: 2026-05-20

## Current State

AEVH-010 through AEVH-040 are complete. AEVH-050 closeout is next.

## Active Task

AEVH-050 - Close lane.

## File Scope

- `apps/android/scripts/Smoke-Emulator.ps1`
- `apps/android/scripts/Smoke-Regression.ps1`
- `apps/android/scripts/Validate-AndroidLocal.ps1`
- `docs/workstreams/android-end-to-end-validation-hardening/`

## Validation

Run script parse checks first. Prefer `Validate-AndroidLocal.ps1 -SkipSmoke`
for no-emulator proof, then run the focused or full emulator-backed gate when
the emulator is healthy.

## Notes

- Do not rewrite the harness in Python in this lane.
- Do not change smoke navigation or UI text assertions unless a gate failure
  proves the existing assertion is stale.
- Do not commit generated `apps/android/build/` evidence artifacts.
- Full default validation remains the closeout gate if emulator/server-backed
  fixture state is healthy.
