# Android Developer Validation Entrypoint - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Gate Set

### No-Emulator Validation

```powershell
pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke
```

This proves the developer entrypoint can run Android JVM tests, optionally
assemble the debug APK, and write a report without requiring an emulator.

### Default Local Validation

```powershell
pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1
```

This is the desired local handoff gate when an emulator is available. It
delegates smoke states to `Smoke-Regression.ps1`.

### Diff Hygiene

```powershell
git diff --check
```

## Evidence Anchors

- `apps/android/scripts/Validate-AndroidLocal.ps1`
- `apps/android/build/validation/<timestamp>/report.md`
- `docs/workstreams/android-developer-validation-entrypoint/TODO.md`
- `docs/workstreams/android-developer-validation-entrypoint/HANDOFF.md`

## Notes

Generated validation logs and reports stay under `apps/android/build/` and
should not be committed by default.

## ADV-020 / ADV-030 Evidence

Validated on 2026-05-19:

```powershell
pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Validate-AndroidLocal.ps1' -Raw)) | Out-Null"
pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1
git diff --check
```

Validation reports:

- No-emulator report:
  `apps/android/build/validation/20260519-094914/report.md`
- Default validation report:
  `apps/android/build/validation/20260519-095005/report.md`

Delegated smoke regression report:

- `apps/android/build/smoke-regression/20260519-095037/report.md`

What this proves:

- The no-emulator command runs Android JVM tests and debug assemble, then
  writes a combined validation report with smoke explicitly skipped.
- The default command runs Android JVM tests, debug assemble, and the stable
  smoke regression state set.
- The validation report links the smoke regression report instead of copying or
  owning smoke state details.
- `git diff --check` passed with Git line-ending normalization warnings for
  edited Windows-tracked files only.

Closeout decision:

- Close this local developer entrypoint lane. CI/device-farm packaging, golden
  screenshot policy, structured JSON/JUnit output, and Python rewrite remain
  follow-ons.
