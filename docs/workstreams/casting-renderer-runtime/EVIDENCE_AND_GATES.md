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

## Task Evidence

### CAST-020 - Readiness And Characterization

Added characterization tests:

- `public_route_inventory_currently_has_no_renderer_session_surface` proves the
  Public Client protocol has Playback Session, cancel, and heartbeat routes,
  but no renderer or cast route surface yet.
- `browser_playback_session_currently_has_no_renderer_session_surface` proves
  browser playback can create a Playback Session and accept heartbeat updates,
  while the public session JSON has no `renderer_session_id`, target, command
  endpoint, control capabilities, or supported commands.

Validation:

```bash
cargo nextest run -p nako-client-protocol public_route_inventory_currently_has_no_renderer_session_surface --no-fail-fast
cargo nextest run -p nako-server browser_playback_session_currently_has_no_renderer_session_surface --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo fmt --all
```

Result:

- Protocol targeted test: 1 passed, 10 skipped.
- Server targeted test: 1 passed, 358 skipped.
- `nako-server playback`: 77 passed, 282 skipped.
- `nako-client-protocol public`: 11 passed.
- `cargo fmt --all`: passed.

Conclusion:

- CAST-030 should add explicit Renderer Session and Renderer Command records.
- Current Playback Session and browser-ticket behavior should remain the
  playback transport baseline, not become the renderer/control model.

### CAST-030 - Renderer Session Domain

Changed behavior:

- Added `RendererSessionId` and `RendererCommandId`.
- Added core `RendererSessionState`, `RendererCommandState`,
  `NewRendererSession`, `RendererSessionRecord`, `RendererSessionHeartbeat`,
  `NewRendererCommand`, `RendererCommandRecord`, and
  `RendererCommandCompletion`.
- Added `RendererSessionRepository` with session upsert/list/heartbeat,
  playback-session attachment, command creation/list, command claim, and
  terminal command completion.
- Added durable SQLite and PostgreSQL baseline tables for `renderer_sessions`
  and `renderer_commands`.
- Added SQLite and PostgreSQL repository adapters through `NakoDatabase`.

Persistence decision:

- Renderer sessions and commands are durable now. This is the stronger boundary
  for Nako-to-Nako command polling, later adapter processes, and future
  multi-process execution. Playback Session and Transcode Session remain
  separate identities.

Validation:

```bash
cargo nextest run -p nako-core renderer --no-fail-fast
cargo nextest run -p nako-db renderer --no-fail-fast
cargo fmt --all -- --check
```

Result:

- `nako-core renderer`: 3 passed, 26 skipped.
- `nako-db renderer`: 1 passed, 152 skipped.
- `cargo fmt --all -- --check`: passed.

Notes:

- The PostgreSQL renderer contract is compiled but ignored by default with the
  existing `NAKO_TEST_POSTGRES_URL` gate. The SQLite contract ran and passed.
- CAST-040 can now add Public Client registration, heartbeat, target listing,
  and command polling/delivery on top of the repository seam.

## Notes

External protocol work should remain adapter-specific and must not expose raw
Source Locators, local paths, bearer tokens, or Transcode Session IDs.
