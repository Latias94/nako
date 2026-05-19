# Android Playback Depth Validation - Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Gate Set

### Focused Playback Depth Smoke

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Emulator.ps1 -FixtureState profile-with-media -SkipAppBuild -SkipFixtureServerBuild
```

This proves the Direct Play depth path against the existing server-backed demo
fixture without rebuilding unchanged artifacts during iteration.

### Script Parse

```powershell
pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Emulator.ps1' -Raw)) | Out-Null"
```

### Diff Hygiene

```powershell
git diff --check
```

## Evidence Anchors

- `apps/android/scripts/Smoke-Emulator.ps1`
- `apps/android/build/smoke/<timestamp>-profile-with-media-<serial>/`
- `apps/android/SMOKE_FIXTURES.md`
- `docs/workstreams/user-playback-state-contract/`

## Notes

- Do not mark playback depth complete from the initial player surface alone.
- Server readback evidence must use Public Client API routes and must be
  token-safe.

## APDV-010 Evidence

Claim: the first playback-depth lane is scoped to Direct Play advancement and
server **User Playback State** readback.

Evidence:

- `DESIGN.md` defines the target state and non-goals.
- `TODO.md` splits playback advancement, server readback, and closeout tasks.
