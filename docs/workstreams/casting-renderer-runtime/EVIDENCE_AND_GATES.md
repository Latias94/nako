# Casting Renderer Runtime - Evidence And Gates

Status: Closed
Last updated: 2026-05-27

## Smallest Current Repro

```bash
cargo nextest run -p nako-server renderer --no-fail-fast
```

This proves the Nako-to-Nako renderer registration, heartbeat, listing,
command delivery, policy-checked play command flow, and Admin renderer
diagnostics.

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
- `docs/workstreams/casting-renderer-runtime/ADAPTER_FOLLOW_ONS.md`
- `docs/workstreams/casting-renderer-runtime/TODO.md`
- `crates/nako-server/src/app/renderer.rs`
- `crates/nako-server/src/app/casting.rs`
- `crates/nako-server/src/http/renderer.rs`
- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-api/src/admin/playback.rs`
- `crates/nako-server/src/http/admin.rs`

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

### CAST-040 - Nako Remote Client Adapter

Changed behavior:

- Added Public Client renderer routes:
  - `GET /renderers`
  - `POST /renderers`
  - `POST /renderers/{renderer_session_id}/heartbeat`
  - `POST /renderers/{renderer_session_id}/commands/next`
  - `POST /renderers/{renderer_session_id}/commands/{command_id}/complete`
- Added protocol-owned renderer DTOs for registration, heartbeat, renderer
  session listing, command polling, and command completion.
- Added `RendererAppService` as the server-side adapter boundary over the
  durable renderer repository. It owns Nako-to-Nako target validation, owner
  checks, TTL expiry, capability normalization, command shape validation, and
  terminal completion rules.
- Updated Public OpenAPI and generated TypeScript/Kotlin SDK outputs so the
  route inventory, OpenAPI document, and SDK package entries remain in sync.

Boundary decision:

- CAST-040 accepts only Nako remote/native renderer targets over bearer auth.
  Chromecast, DLNA, and AirPlay remain future Renderer Adapters; they cannot
  register through the Nako-to-Nako Public Client route.
- Public renderer DTOs do not expose owner principals, raw Source Locators,
  local paths, bearer tokens, or command payload JSON.

Validation:

```bash
cargo nextest run -p nako-server renderer --no-fail-fast
cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result:

- `nako-server renderer`: 3 passed, 358 skipped.
- `nako-server playback/renderer`: 79 passed, 282 skipped.
- `nako-client-protocol public`: 11 passed.
- `nako-api admin_contract/public_openapi/sdk`: 21 passed, 39 skipped.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.

Notes:

- The server renderer gate also runs
  `browser_playback_session_currently_has_no_renderer_session_surface`, which
  continues to prove Playback Session JSON remains separate from Renderer
  Session/control state.
- CAST-050 should add the controller-to-renderer play command flow through the
  existing policy-aware Playback App Service instead of bypassing playback
  policy.

### CAST-050 - Cast Play Command Flow

Changed behavior:

- Added `CastingAppService` as the orchestration boundary for controller
  commands that need both Renderer Session state and Playback App Service
  policy/session behavior.
- Added `PlaybackAppService::start_renderer_playback_session`, which plans
  against the renderer's registered target/media capabilities and enforces
  `remote_control`, cast, remote playback, and direct-play policy before any
  Playback Session is created.
- Added Public Client route
  `POST /renderers/{renderer_session_id}/commands/play`.
- The allowed path creates a direct-play Playback Session, queues a
  `play` Renderer Command with `source_id`, `item_id`, `playback_session_id`,
  and optional `position_ms`, then attaches the Playback Session to the
  Renderer Session.
- The denied path proves playback policy/control denial creates no Playback
  Session, Transcode Session, browser ticket, cast ticket, or Renderer Command.

Boundary decision:

- CAST-050 intentionally accepts only direct-play renderer command starts.
  If the renderer's capabilities require remux or HLS, Nako returns
  `Unsupported` instead of creating a placeholder Playback Session without the
  corresponding transport. Remux/HLS renderer transport should be designed with
  cast-safe URLs and target-specific adapter behavior in CAST-060 follow-ons.

Validation:

```bash
cargo nextest run -p nako-server renderer --no-fail-fast
cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result:

- `nako-server renderer`: 5 passed, 358 skipped.
- `nako-server playback/renderer`: 81 passed, 282 skipped.
- `nako-client-protocol public`: 11 passed.
- `nako-api admin_contract/public_openapi/sdk`: 21 passed, 39 skipped.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.

Notes:

- Nako-to-Nako play control now uses the same effective playback policy seam as
  normal playback, not a renderer-specific bypass.
- CAST-060 can add redaction-safe Admin diagnostics and split external
  Chromecast, DLNA, AirPlay, and non-direct Nako renderer transport work.

### CAST-060 - Diagnostics And External Adapter Follow-Ons

Changed behavior:

- Added Admin route `GET /admin/v1/playback/renderers`.
- Added `AdminRendererRuntimeDiagnosticsResponse` with renderer runtime
  readiness, session counts, safe session summaries, and adapter readiness.
- Added Admin Web generated contract route/type coverage for
  `playbackRenderers`.
- Added `ADAPTER_FOLLOW_ONS.md` with concrete adapter contracts for Nako
  non-direct renderer transport, Chromecast, DLNA renderer, and AirPlay.
- Updated ADR 0040 to state that Admin diagnostics are part of the casting
  boundary and that planned external adapters are not runtime failures.

Boundary decision:

- `nako_remote_client` with bearer auth is the only ready adapter today.
- Non-direct Nako renderer transport, Chromecast, DLNA, and AirPlay are exposed
  as planned adapter readiness entries so Admin Web can show roadmap state
  without misreporting them as broken dependencies.
- Diagnostics intentionally exclude owner principals, raw capability JSON,
  command payload JSON, source locators, local paths, bearer tokens, cast ticket
  material, and protocol-private network addresses.

Validation:

```bash
cargo nextest run -p nako-server admin_v1_playback_renderers_reports_safe_diagnostics_and_adapter_readiness --no-fail-fast
cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts
cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk)' --no-fail-fast
cargo nextest run -p nako-server renderer --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk) | test(admin_renderer_runtime_diagnostics)' --no-fail-fast
```

Result:

- Targeted Admin renderer diagnostics test: 1 passed, 363 skipped.
- Admin contract generation: passed.
- API admin/openapi/sdk gate: 21 passed, 39 skipped.
- Server renderer gate: 6 passed, 358 skipped.
- API admin/openapi/sdk plus renderer DTO redaction gate: 22 passed, 39
  skipped.

Notes:

- CAST-070 should close this lane unless review finds a blocking issue.
- Protocol-specific follow-ons should start from
  `ADAPTER_FOLLOW_ONS.md` instead of adding protocol branches to playback
  routes.

### CAST-070 - Closeout

Review result:

- Workstream Compliance: no blocking findings. Renderer Sessions, command
  lifecycle, Nako-to-Nako adapter routes, policy-checked play command flow,
  Admin renderer diagnostics, and adapter follow-on contracts match the lane
  target.
- Code Quality: no blocking findings. Runtime orchestration stays in
  `RendererAppService` and `CastingAppService`; Admin HTTP only maps safe
  diagnostics; protocol-specific work is not mixed into playback routes.
- Missing Gates: none for this lane.
- Residual Risk: non-direct Nako renderer transport, Chromecast, DLNA, and
  AirPlay are not implemented; they are split in `ADAPTER_FOLLOW_ONS.md`.

Final validation:

```bash
cargo nextest run -p nako-server -E 'test(playback) | test(renderer)' --no-fail-fast
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo fmt --all -- --check
git diff --check
python -m json.tool docs/workstreams/casting-renderer-runtime/WORKSTREAM.json
```

Result:

- `nako-server playback/renderer`: 82 passed, 282 skipped.
- `nako-client-protocol public`: 11 passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- `WORKSTREAM.json`: parsed.

## Notes

External protocol work should remain adapter-specific and must not expose raw
Source Locators, local paths, bearer tokens, or Transcode Session IDs.
