# Android End-To-End Validation Hardening - Evidence And Gates

Status: Closed
Last updated: 2026-05-20

## Gates

Script parse gate:

```powershell
pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Emulator.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Regression.ps1' -Raw)) | Out-Null; [scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Validate-AndroidLocal.ps1' -Raw)) | Out-Null"
```

No-emulator validation gate:

```powershell
pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
```

Focused smoke gate:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media -SkipAppBuild -SkipFixtureServerBuild
```

Full local validation gate:

```powershell
pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1
```

Diff hygiene:

```powershell
git diff --check
```

## Evidence

- AEVH-010: Workstream opened on 2026-05-20. Existing smoke/developer
  validation lanes are closed and referenced as the baseline.
- AEVH-020: Script parse gate passed on 2026-05-20 after adding state-level
  `report.json` output to `Smoke-Emulator.ps1`.
- AEVH-030: Controlled failure run passed its evidence-shape check on
  2026-05-20:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup -Serial not-a-device -SkipBuild -RetriesPerState 0 -ContinueOnFailure`.
  The command intentionally returned failure because the serial is invalid, but
  wrote `apps/android/build/smoke-regression/20260520-105616/report.json` with
  stable `report_markdown` and `report_json` fields set to null for the failed
  state.
- AEVH-030: Focused success run passed on 2026-05-20:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup -SkipBuild -RetriesPerState 0`.
  Evidence: `apps/android/build/smoke-regression/20260520-105821/report.md`
  and `apps/android/build/smoke-regression/20260520-105821/report.json`.
  The regression JSON links the state Markdown and structured report.
- AEVH-040: No-emulator validation passed on 2026-05-20:
  `pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke`.
  Evidence: `apps/android/build/validation/20260520-105656/report.md` and
  `apps/android/build/validation/20260520-105656/report.json`.
- AEVH-040: `git diff --check` passed on 2026-05-20 with only Git line-ending
  normalization warnings for edited PowerShell files.
- AEVH-050: Focused media smoke passed on 2026-05-20:
  `pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States profile-with-media -SkipBuild -RetriesPerState 0`.
  Evidence: `apps/android/build/smoke-regression/20260520-112424/report.md`
  and `apps/android/build/smoke-regression/20260520-112424/report.json`.
- AEVH-050: Full default validation passed on 2026-05-20:
  `pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1`.
  Evidence: `apps/android/build/validation/20260520-112917/report.md` and
  `apps/android/build/validation/20260520-112917/report.json`.
  Delegated regression evidence:
  `apps/android/build/smoke-regression/20260520-112949/report.md` and
  `apps/android/build/smoke-regression/20260520-112949/report.json`.

## Closeout Notes

- `profile-with-media` and `profile-active-remux` now prepare the demo fixture
  under the current smoke evidence directory. This prevents stale local
  `apps/android/build/demo-fixtures/server-backed` databases from failing
  future runs when migration checksums change.
- Direct Play completion now expects the detail surface to expose `Play` after
  server readback marks the item watched and clears Continue Watching.
- Cast & Crew smoke evidence now scrolls to the actual person row before
  asserting person route text, avoiding a brittle heading-only viewport.

## Notes

Generated validation logs, screenshots, UI hierarchy dumps, and reports remain
under `apps/android/build/` and should not be committed.
