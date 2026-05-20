# Android JUnit Validation Reports - Evidence And Gates

Status: Active
Last updated: 2026-05-20

## Gates

Script parse gate:

```powershell
pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Regression.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Validate-AndroidLocal.ps1' -Raw)) | Out-Null"
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

## Notes

- Generated JUnit XML reports stay under `apps/android/build/`.
- JUnit XML must not include bearer tokens, raw source locators, screenshot
  binaries, or full UI hierarchy payloads.
- Prefer implementation helpers inside the existing PowerShell scripts. Do not
  rewrite the validation harness in Python in this lane.
