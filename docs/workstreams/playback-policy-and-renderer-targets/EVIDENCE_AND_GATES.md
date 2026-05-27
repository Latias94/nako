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

## Notes

Fresh verification is required before marking a task, Codex goal, or lane
complete. Jellyfin reference evidence is used only for behavior pressure and
architecture comparison; no reference source is copied into Nako.
