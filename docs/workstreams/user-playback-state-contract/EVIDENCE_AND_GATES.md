# User Playback State Contract Evidence And Gates

Status: Active
Last updated: 2026-05-19

## Smallest Current Repro

Current gap: no public route exists for server-authoritative **User Playback
State** lookup/reporting, and Android only has device-local resume.

Useful reads:

```powershell
rg -n "User Playback State|Continue Watching|DevicePlayback|resume|watched|progress" CONTEXT.md docs apps/android crates -g '*.md' -g '*.kt' -g '*.rs'
```

## Gate Set

### Contract Gate

```powershell
git diff --check
```

Proves workstream docs and API contract edits are clean before implementation.

### Server Gate

```powershell
cargo nextest run -p taru-db -p taru-server user_playback_state --no-fail-fast
```

Proves storage, repository, principal scoping, and app-service behavior.

### API And SDK Gate

```powershell
cargo nextest run -p taru-api -p taru-client --no-fail-fast
npm run check --prefix sdk/typescript
```

Proves public DTO/OpenAPI/SDK surfaces agree.

### Android Gate

```powershell
apps/android/gradlew.bat -p apps/android :app:testDebugUnitTest --no-daemon
```

Proves Android client/UI behavior and local fallback boundaries.

### Smoke Gate

```powershell
pwsh -NoProfile -File apps/android/scripts/Smoke-Regression.ps1 -States profile-with-media
git diff --check
```

Proves the emulator/server-backed user-facing path.

## Evidence Anchors

- `CONTEXT.md`
- `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- `docs/api/HTTP_API.md`
- `docs/workstreams/user-playback-state-contract/CONTRACT.md`
- `docs/workstreams/android-client-foundation/CLIENT_INTERFACE_DESIGN.md`
- `docs/workstreams/android-device-local-playback-position/`
- `docs/workstreams/android-public-client-api-coverage/`
- `crates/taru-client-protocol/`
- `crates/taru-api/`
- `crates/taru-client/`
- `crates/taru-server/`
- `apps/android/`

## Notes

- Do not mark Continue Watching complete from Android local storage.
- Do not store raw source locators, filesystem paths, tokens, or playback
  session internals in user playback state DTOs.
- Fresh verification is required before marking a task, Codex goal, or lane
  complete.

## UPS-010 Evidence

Claim: the first public **User Playback State** contract and **Single-Admin
Mode** principal semantics are frozen for implementation.

Evidence:

- `CONTRACT.md` defines `/users/me/playback-state/...` routes, DTO names,
  progress semantics, watched threshold policy, Continue Watching semantics, and
  first-slice deferrals.
- ADR-0028 defines explicit principal resolution and forbids treating bearer
  tokens or global item rows as user playback state.
- `DESIGN.md` links the frozen contract and updates the workstream from draft
  planning to active implementation.

Fresh gate evidence:

- 2026-05-19: `git diff --check` - PASS. This proves the UPS-010 ADR,
  contract, task ledger, and handoff edits have no whitespace errors.
