# Playback Policy And Renderer Targets - Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Smallest Current Repro

```bash
cargo nextest run -p nako-server playback --no-fail-fast
```

This proves the current Public Client playback routes and app service behavior
before policy-aware enforcement changes.

## Gate Set

### Design Gate

```bash
python -m json.tool docs/workstreams/playback-policy-and-renderer-targets/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-policy-and-renderer-targets docs/adr/0039-playback-policy-and-renderer-target-boundary.md
```

### Planner Gate

```bash
cargo nextest run -p nako-playback --no-fail-fast
```

This proves pure planner policy/target behavior without server repository or
HTTP dependencies.

### Core Policy Gate

```bash
cargo nextest run -p nako-core playback --no-fail-fast
```

This proves shared domain records and policy defaults if the implementation
touches `nako-core`.

### Server Playback Gate

```bash
cargo nextest run -p nako-server playback --no-fail-fast
```

This proves route/app behavior, session creation, ticket validation, and
transcode artifact behavior.

### API Contract Gate

```bash
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast
```

This proves safe Public Client DTOs, generated SDK/OpenAPI shape, and Admin
contract updates.

### Closeout Gate

```bash
cargo fmt --all -- --check
git diff --check
python -m json.tool docs/workstreams/playback-policy-and-renderer-targets/WORKSTREAM.json
```

Use narrower closeout if the workspace is too large, and record the reason.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or in `HANDOFF.md`.

## Evidence Anchors

- `docs/adr/0039-playback-policy-and-renderer-target-boundary.md`
- `docs/workstreams/playback-policy-and-renderer-targets/DESIGN.md`
- `docs/workstreams/playback-policy-and-renderer-targets/TODO.md`
- `crates/nako-playback/src/lib.rs`
- `crates/nako-server/src/app/playback/`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-api/src/public_client.rs`

## Task Evidence

### PRT-020 - Current Behavior Characterization

Added characterization tests:

- `browser_ticket_play_access_currently_allows_all_playback_modes` proves a
  viewer with `LibraryAccessLevel::Play` can currently request direct, remux,
  and HLS browser playback tickets without mode-specific policy.
- `remux_source_currently_starts_without_principal_or_playback_policy` proves
  the app-service remux path currently starts from source/client/container
  facts without principal or effective playback policy input.
- `planner_characterizes_remote_context_as_not_a_permission_gate_yet` proves
  `remote=true` is currently profile identity/context, not an allow/deny
  permission gate.

Validation:

```bash
cargo nextest run -p nako-playback planner_characterizes_remote_context_as_not_a_permission_gate_yet --no-fail-fast
cargo nextest run -p nako-server -E 'test(browser_ticket_play_access_currently_allows_all_playback_modes) | test(remux_source_currently_starts_without_principal_or_playback_policy)' --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-playback --no-fail-fast
```

Result:

- `nako-server playback`: 71 passed, 282 skipped.
- `nako-playback`: 8 passed.

### PRT-030 - Policy And Target Domain Records

Added pure records:

- `crates/nako-core/src/playback_policy.rs`: playback permission vocabulary,
  mode-specific denial reasons, `PlaybackPermissionPolicy`, and
  `EffectivePlaybackPolicy`.
- `crates/nako-core/src/playback_target.rs`: target kind, network scope,
  transport auth, renderer control command, and renderer control capability
  vocabulary.
- `PlaybackTargetId` in `nako-core` IDs.
- `crates/nako-playback/src/lib.rs`: `PlaybackTarget` combining target
  vocabulary with `ClientPlaybackCapabilities`, plus re-exports needed by the
  planner crate.

Validation:

```bash
cargo fmt --all -- --check
cargo nextest run -p nako-core playback --no-fail-fast
cargo nextest run -p nako-playback --no-fail-fast
```

Result:

- `nako-core playback`: 7 passed, 17 skipped.
- `nako-playback`: 10 passed.

### PRT-040 - Planner Enforcement

Changed planner/API behavior:

- `PlaybackPlanningRequest` now receives `PlaybackTarget` and
  `EffectivePlaybackPolicy`.
- `PlaybackDecision` can represent an internal denied plan with
  `PlaybackMode::Denied`, `PlaybackExecutionPlan::Denied`, and `PlaybackDenial`.
- Planner denies direct, remux, transcode, remote, and cast paths when effective
  policy rejects the required permission.
- Server playback app still passes current default policy/target behavior; real
  user/role/settings policy resolution is PRT-050.
- Public API maps denial to safe `denied` mode and `policy_denied` reason.
- TypeScript and Kotlin Public SDK package entries were regenerated.

Validation:

```bash
cargo fmt --all -- --check
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-api public --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk)' --no-fail-fast
cargo nextest run -p nako-client-protocol public --no-fail-fast
```

Result:

- `nako-playback`: 15 passed.
- `nako-api public`: 22 passed, 37 skipped.
- `nako-server playback`: 71 passed, 282 skipped.
- `nako-api public_openapi|sdk`: 16 passed, 43 skipped.
- `nako-client-protocol public`: 10 passed.

Additional attempted gate:

```bash
npm --prefix sdk/typescript run check
```

Result: failed because `tsc` was not installed or available in the local
environment; no TypeScript type error was reported.

## Notes

Fresh verification is required before marking a task, Codex goal, or lane
complete. Jellyfin reference evidence is used only for behavior pressure and
architecture comparison; no reference source is copied into Nako.
