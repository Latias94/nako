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

## Notes

Fresh verification is required before marking a task, Codex goal, or lane
complete. Jellyfin reference evidence is used only for behavior pressure and
architecture comparison; no reference source is copied into Nako.
