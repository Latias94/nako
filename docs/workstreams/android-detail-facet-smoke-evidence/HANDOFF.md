# Android Detail Facet Smoke Evidence - Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

Lane is closed. ADF-010, ADF-020, and ADF-030 are complete.

## Completed Slice

ADF-020: make `profile-with-media` smoke prove detail-page Genre, Tag, and
Person navigation to server-backed facet result routes.

## File Scope

- `apps/android/scripts/Smoke-Emulator.ps1`
- `apps/android/SMOKE_FIXTURES.md`
- `apps/android/README.md`
- Workstream docs under this directory.

## Validation

Passed:

```powershell
pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media
pwsh -NoProfile -File apps\android\scripts\Smoke-Regression.ps1 -States empty-setup,profile-missing-token,profile-with-media -SkipBuild
git diff --check
```

Latest focused smoke report:

`apps/android/build/smoke/20260519-104315-profile-with-media-emulator-5554/report.md`

Latest regression smoke report:

`apps/android/build/smoke-regression/20260519-104729/report.md`

## Notes

- Reused existing `Night Harbor` fixture metadata: `Mystery`, `Lighthouse`,
  and `Mira Vale`.
- Unsupported relationship families remain explicit API gaps.
- Generated smoke evidence under `apps/android/build` is not committed.
- Untracked `output/` and `tmp/` were not touched.
- Follow-ons: Collection, Studio, Series/Hierarchy, Year, Item Kind, and richer
  role-specific navigation.
