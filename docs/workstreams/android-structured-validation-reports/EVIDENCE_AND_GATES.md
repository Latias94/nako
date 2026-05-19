# Android Structured Validation Reports - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Gate Set

### Script Parse

```powershell
pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Regression.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Validate-AndroidLocal.ps1' -Raw)) | Out-Null"
```

### Smoke JSON

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup -SkipBuild -RetriesPerState 0
pwsh -NoProfile -Command "Get-Content -Raw '<report.json>' | ConvertFrom-Json | Out-Null"
```

### Local Validation JSON

```powershell
pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke
pwsh -NoProfile -Command "Get-Content -Raw '<report.json>' | ConvertFrom-Json | Out-Null"
```

### Diff Hygiene

```powershell
git diff --check
```

## Evidence Anchors

- `apps/android/scripts/Smoke-Regression.ps1`
- `apps/android/scripts/Validate-AndroidLocal.ps1`
- `apps/android/build/smoke-regression/<timestamp>/report.json`
- `apps/android/build/validation/<timestamp>/report.json`

## Notes

- JSON reports are generated local artifacts and should not be committed.
- JSON report content must be token-safe.

## ASVR-010 Evidence

Claim: the structured report lane is scoped as additive JSON output for the
existing Android validation commands.

Evidence:

- `DESIGN.md` defines target state, report seam, and non-goals.
- `TODO.md` splits smoke JSON, validation JSON, and closeout tasks.

## ASVR-020 Evidence

Claim: `Smoke-Regression.ps1` writes a stable JSON report in addition to the
existing Markdown report.

Evidence:

- `apps/android/scripts/Smoke-Regression.ps1` writes `report.json`, prints
  `Structured report: ...`, and keeps `report.md` behavior compatible.
- `apps/android/SMOKE_FIXTURES.md` documents the JSON report artifact.

Fresh gate evidence:

- 2026-05-19: `pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup -SkipBuild -RetriesPerState 0 -OutputRoot apps/android/build/smoke-regression-asvr` - PASS. Report: `apps/android/build/smoke-regression-asvr/20260519-171154/report.json`.
- 2026-05-19: `pwsh -NoProfile -Command "Get-Content -LiteralPath 'apps/android/build/smoke-regression-asvr/20260519-171154/report.json' -Raw | ConvertFrom-Json | Out-Null"` - PASS.

## ASVR-030 Evidence

Claim: `Validate-AndroidLocal.ps1` writes a stable JSON report and can point to
delegated smoke JSON when smoke runs.

Evidence:

- `apps/android/scripts/Validate-AndroidLocal.ps1` writes `report.json`, prints
  `Structured report: ...`, and includes `delegated_reports.smoke_json`.
- The script now normalizes relative `-OutputRoot` paths before Gradle changes
  the current directory.
- `apps/android/README.md` documents validation JSON output.

Fresh gate evidence:

- 2026-05-19: `pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke -OutputRoot apps/android/build/validation-asvr` - PASS. Report: `apps/android/build/validation-asvr/20260519-170805/report.json`.
- 2026-05-19: `pwsh -NoProfile -Command "Get-Content -LiteralPath 'apps/android/build/validation-asvr/20260519-170805/report.json' -Raw | ConvertFrom-Json | Out-Null"` - PASS.
- 2026-05-19: `git diff --check` - PASS.

## ASVR-040 Evidence

Claim: the structured validation report lane is complete.

Closeout decision:

- Close this lane. JUnit XML export, CI upload integration, and golden visual
  diffing remain separate follow-ons.
