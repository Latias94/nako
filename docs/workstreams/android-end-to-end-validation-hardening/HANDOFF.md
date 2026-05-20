# Android End-To-End Validation Hardening - Handoff

Status: Closed
Last updated: 2026-05-20

## Current State

AEVH-010 through AEVH-050 are complete. The lane is closed.

## Active Task

None.

## File Scope

- `apps/android/scripts/Smoke-Emulator.ps1`
- `apps/android/scripts/Smoke-Regression.ps1`
- `apps/android/scripts/Validate-AndroidLocal.ps1`
- `docs/workstreams/android-end-to-end-validation-hardening/`

## Validation

Closeout evidence was recorded on 2026-05-20:

- script parse gate passed;
- `Validate-AndroidLocal.ps1 -SkipSmoke` passed;
- focused `profile-with-media` smoke regression passed;
- full default `Validate-AndroidLocal.ps1` passed;
- `git diff --check` passed with line-ending normalization warnings only.

Final evidence:

- `apps/android/build/validation/20260520-112917/report.md`
- `apps/android/build/validation/20260520-112917/report.json`
- `apps/android/build/smoke-regression/20260520-112949/report.md`
- `apps/android/build/smoke-regression/20260520-112949/report.json`

## Notes

- Do not rewrite the harness in Python in this lane.
- Do not commit generated `apps/android/build/` evidence artifacts.
- Media smoke fixture data is now prepared under each smoke evidence directory,
  so stale shared local fixture databases cannot fail future runs after
  migration checksum changes.
- Direct Play completion intentionally returns to a detail page with `Play`
  instead of `Resume` after server readback marks the item watched and clears
  Continue Watching.
- Future CI packaging and screenshot golden diffing are separate follow-on
  scopes, not residual work for this lane.
