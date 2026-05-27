# Casting Renderer Runtime - Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Smallest Current Repro

```bash
cargo nextest run -p nako-server playback --no-fail-fast
```

This proves the current Playback Session and playback route behavior before
Renderer Session and command delivery are added.

## Gate Set

### Design Gate

```bash
python -m json.tool docs/workstreams/casting-renderer-runtime/WORKSTREAM.json
git diff --check -- docs/workstreams/casting-renderer-runtime docs/adr/0040-casting-as-renderer-session-adapter.md
```

### Renderer Domain Gate

```bash
cargo nextest run -p nako-core renderer --no-fail-fast
cargo nextest run -p nako-db renderer --no-fail-fast
```

Use only after renderer records/repositories exist.

### Server Renderer Gate

```bash
cargo nextest run -p nako-server renderer --no-fail-fast
```

This proves renderer registration, heartbeat, command delivery, authorization,
and diagnostics.

### Playback Integration Gate

```bash
cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast
```

This proves cast play commands integrate with the policy-aware Playback App
Service and do not regress normal playback routes.

### Public/Admin API Gate

```bash
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk)' --no-fail-fast
```

This proves safe DTOs and generated contract behavior.

### Closeout Gate

```bash
cargo fmt --all -- --check
git diff --check
python -m json.tool docs/workstreams/casting-renderer-runtime/WORKSTREAM.json
```

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or in `HANDOFF.md`.

## Evidence Anchors

- `docs/adr/0040-casting-as-renderer-session-adapter.md`
- `docs/workstreams/casting-renderer-runtime/DESIGN.md`
- `docs/workstreams/casting-renderer-runtime/TODO.md`
- future renderer runtime modules
- future Public Client renderer DTOs
- future Admin renderer diagnostics DTOs

## Dependency Evidence

- `playback-policy-and-renderer-targets` closed on 2026-05-27.
- Effective playback policy and `PlaybackTarget` are available to reuse.
- Safe Public playback target/denial DTOs and Admin policy diagnostics exist,
  so this lane can focus on Renderer Sessions, commands, and adapters.

## Notes

External protocol work should remain adapter-specific and must not expose raw
Source Locators, local paths, bearer tokens, or Transcode Session IDs.
