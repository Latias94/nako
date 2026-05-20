# Android JUnit Validation Reports - Evidence And Gates

Status: Active
Last updated: 2026-05-20

## Gates

Script parse gate:

```powershell
pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Android-JUnitReport.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Regression.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Validate-AndroidLocal.ps1' -Raw)) | Out-Null"
```

Focused smoke report gate:

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States empty-setup -SkipBuild -RetriesPerState 0
```

Focused local validation gate:

```powershell
pwsh -NoProfile -File apps/android/scripts/Validate-AndroidLocal.ps1 -SkipSmoke
```

Generated XML parse gate:

```powershell
pwsh -NoProfile -Command "[xml](Get-Content -LiteralPath '<report.junit.xml>' -Raw) | Out-Null"
```

Diff hygiene:

```powershell
git diff --check
```

## Evidence

- AJVR-010 opened on 2026-05-20:
  - Scope is additive JUnit XML output for existing Android validation scripts.
  - JSON and Markdown report contracts remain compatible.
  - CI upload/artifact retention and golden visual diffing are explicit
    follow-ons.
- AJVR-010 completed on 2026-05-20:
  - `DESIGN.md` freezes `report.junit.xml`, `<testsuites>`, suite names,
    testcase names, pass/fail/skipped mapping, allowed properties, and
    token-safety constraints.
  - Fresh validation:
    `Get-Content -LiteralPath 'docs/workstreams/android-junit-validation-reports/WORKSTREAM.json' -Raw | ConvertFrom-Json | Out-Null`
    passed.
  - Fresh validation:
    `git diff --check` passed.
- AJVR-020 completed on 2026-05-20:
  - `Smoke-Regression.ps1` writes `report.junit.xml`, prints `JUnit report:`,
    and includes `report_junit` in `report.json`.
  - JUnit XML uses suite `taru.android.smoke-regression`, testcase classname
    `taru.android.smoke`, `step.android-build`, and one `state.<name>` testcase
    per requested smoke state.
  - Fresh validation:
    `pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Regression.ps1' -Raw)) | Out-Null"`
    passed.
  - Fresh validation:
    `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup -SkipBuild -RetriesPerState 0 -OutputRoot apps\android\build\smoke-regression-ajvr`
    passed.
  - Fresh validation:
    `[xml](Get-Content -LiteralPath 'apps/android/build/smoke-regression-ajvr/20260520-201331/report.junit.xml' -Raw) | Out-Null`
    passed.
  - Fresh validation:
    `git diff --check` passed.
  - Generated evidence:
    `apps/android/build/smoke-regression-ajvr/20260520-201331/report.junit.xml`.
- AJVR-030 completed on 2026-05-20:
  - Added shared `Android-JUnitReport.ps1` helpers used by both smoke
    regression and local validation report adapters.
  - `Validate-AndroidLocal.ps1` writes `report.junit.xml`, prints
    `JUnit report:`, includes `report_junit` in `report.json`, and links
    delegated `smoke_junit` when smoke runs.
  - JUnit XML uses suite `taru.android.local-validation`, testcase classname
    `taru.android.validation`, and step testcases `step.android-unit-tests`,
    `step.android-build`, and `step.smoke-regression`.
  - Fresh validation:
    `pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Android-JUnitReport.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Regression.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Validate-AndroidLocal.ps1' -Raw)) | Out-Null"`
    passed.
  - Fresh validation:
    `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup -SkipBuild -RetriesPerState 0 -OutputRoot apps\android\build\smoke-regression-ajvr`
    passed after the shared helper extraction.
  - Fresh validation:
    `[xml](Get-Content -LiteralPath 'apps/android/build/smoke-regression-ajvr/20260520-212823/report.junit.xml' -Raw) | Out-Null`
    passed.
  - Fresh validation:
    `pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke -OutputRoot apps\android\build\validation-ajvr`
    passed.
  - Fresh validation:
    `[xml](Get-Content -LiteralPath 'apps/android/build/validation-ajvr/20260520-212603/report.junit.xml' -Raw) | Out-Null`
    passed.
  - Fresh validation:
    `git diff --check` passed.
  - Generated evidence:
    `apps/android/build/smoke-regression-ajvr/20260520-212823/report.junit.xml`.
  - Generated evidence:
    `apps/android/build/validation-ajvr/20260520-212603/report.junit.xml`.

## Notes

- Generated JUnit XML reports stay under `apps/android/build/`.
- JUnit XML must not include bearer tokens, raw source locators, screenshot
  binaries, or full UI hierarchy payloads.
- Prefer implementation helpers inside the existing PowerShell scripts. Do not
  rewrite the validation harness in Python in this lane.
