# Android Developer Validation Entrypoint - Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

This lane is closed. `ADV-010`, `ADV-020`, and `ADV-030` are complete.

## Closed Task

- Task ID: ADV-030
- Owner: codex
- Files: `apps/android/scripts`, `apps/android/README.md`,
  `docs/workstreams/android-developer-validation-entrypoint`
- Validation:
  `pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke`
  and `git diff --check`; run default validation when an emulator is available.
- Status: DONE
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions

- Keep the entrypoint as a thin wrapper around Gradle and
  `Smoke-Regression.ps1`.
- Default mode should provide handoff confidence.
- `-SkipSmoke` is the no-emulator path.
- Generated reports stay local under `apps/android/build/validation/`.

## Blockers

- None known. Default validation requires an emulator in `adb devices` state.

## Next Recommended Action

- Continue Android product work using
  `pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1` as
  the default local handoff gate.

## Latest Evidence

Validation passed on 2026-05-19:

- `pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Validate-AndroidLocal.ps1' -Raw)) | Out-Null"`
- `pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke`
- `pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1`
- `git diff --check`

Reports:

- `apps/android/build/validation/20260519-094914/report.md`
- `apps/android/build/validation/20260519-095005/report.md`
- `apps/android/build/smoke-regression/20260519-095037/report.md`

## Residual Risks And Follow-ons

- CI/device-farm execution is still out of scope.
- Golden screenshot diffing remains out of scope.
- Structured JSON/JUnit export can be added later if CI needs machine-readable
  validation output.
- Python remains unnecessary for the current local wrapper.
