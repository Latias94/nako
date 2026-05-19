# Android Playback Depth Validation - Evidence And Gates

Status: Closed
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

## APDV-020 Evidence

Claim: `profile-with-media` smoke now proves Direct Play advances beyond the
seeded server resume point.

Evidence:

- `apps/android/scripts/Smoke-Emulator.ps1` waits for `Ended` after starting
  resume playback.
- `player.criteria.txt` now requires `Server resume 0:01`, `00:02`, and
  `Ended`.

Fresh gate evidence:

- 2026-05-19: `pwsh -NoProfile -File apps/android/scripts/Smoke-Emulator.ps1 -FixtureState profile-with-media -SkipAppBuild -SkipFixtureServerBuild -OutputRoot apps/android/build/smoke-apdv` - PASS. Evidence directory: `apps/android/build/smoke-apdv/20260519-174200-profile-with-media-emulator-5554`.
- 2026-05-19: `apps/android/build/smoke-apdv/20260519-174200-profile-with-media-emulator-5554/player.criteria.txt` - PASS, including `00:02` and `Ended`.

## APDV-030 Evidence

Claim: after player exit, smoke proves the server received the playback exit
report through **User Playback State**.

Evidence:

- `apps/android/scripts/Smoke-Emulator.ps1` writes
  `profile-with-media-server-readback.txt` after returning from the player.
- The readback loops until server state is `watched=true` and Continue Watching
  returns zero rows.
- `apps/android/SMOKE_FIXTURES.md` and `apps/android/README.md` document the
  playback-depth readback artifact.

Fresh gate evidence:

- 2026-05-19: `apps/android/build/smoke-apdv/20260519-174200-profile-with-media-emulator-5554/profile-with-media-server-readback.txt` - PASS, observed `watched=True` and `Observed continue-watching rows: 0`.

## APDV-040 Evidence

Claim: the first Direct Play depth validation lane is complete.

Closeout decision:

- Close this lane. HLS/remux/session cancellation, longer watched-threshold
  media, and playback quality checks remain follow-ons.
