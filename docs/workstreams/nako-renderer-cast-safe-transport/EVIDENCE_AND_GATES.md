# Nako Renderer Cast-Safe Transport Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Evidence Log

### NRCT-010

Completed on 2026-05-27.

Evidence:

- opened `docs/workstreams/nako-renderer-cast-safe-transport/`;
- added ADR 0041 for renderer cast-safe transport tickets;
- updated ADR and workstream indexes;
- documented `NRCT-020` as the first executable code task.

Gates:

```powershell
python -m json.tool docs/workstreams/nako-renderer-cast-safe-transport/WORKSTREAM.json
git diff --check -- docs/workstreams/nako-renderer-cast-safe-transport docs/adr/0041-renderer-cast-safe-transport-tickets.md docs/adr/README.md docs/workstreams/README.md
rg -n "投屏|中文|。|，|：|；|（|）" docs\workstreams\nako-renderer-cast-safe-transport docs\adr\0041-renderer-cast-safe-transport-tickets.md
```

Results:

- JSON parsed.
- Diff check passed.
- Non-English punctuation/content check had no matches.

### NRCT-020

Completed on 2026-05-27.

Evidence:

- added Public HTTP characterization tests proving renderer remux decisions are
  currently rejected before runtime records are created;
- added Public HTTP characterization tests proving renderer HLS decisions are
  currently rejected before runtime records are created;
- added Public HTTP characterization for the current `nako_remote_client +
  cast_ticket` registration rejection;
- added browser ticket response characterization proving it carries no
  Renderer Session, Playback Session, renderer command, network scope, or cast
  ticket transport scope.

Gates:

```powershell
cargo nextest run -p nako-server -E 'test(renderer_play_command_currently_rejects_remux_decision_without_runtime_records) | test(renderer_play_command_currently_rejects_hls_decision_without_runtime_records) | test(public_renderer_registration_currently_rejects_nako_remote_cast_ticket_transport) | test(browser_playback_ticket_response_currently_has_no_renderer_transport_scope)' --no-fail-fast
cargo nextest run -p nako-server -E 'test(renderer) | test(playback_ticket) | test(playback)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Results:

- targeted new-test run: 4 passed, 364 skipped.
- focused renderer/playback gate: 86 passed, 282 skipped.
- format check passed.
- diff check passed.

### NRCT-030

Completed on 2026-05-27.

Evidence:

- added `RendererTransportTicketService` as a focused server app module;
- added issue/validate commands with scope binding for principal, Renderer
  Session, Playback Session, Media Source, playback mode, network scope, and
  expiry;
- stored only hashed ticket tokens and kept issued-ticket Debug output
  redaction-safe;
- validated expiry cleanup and scope mismatch failures;
- intentionally did not inject the service into `NakoAppServices` until the
  command transport envelope and playback flow need it, avoiding unused
  composition state.

Gates:

```powershell
cargo nextest run -p nako-server renderer_transport_ticket --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Results:

- renderer transport ticket tests: 2 passed, 368 skipped.
- format check passed.
- diff check passed.

### NRCT-040

Completed on 2026-05-27.

Evidence:

- added typed Public Client renderer transport DTOs for transport mode, URL
  kind, envelope, and URL entries;
- attached optional `transport` to `RendererCommandDto` so both play responses
  and renderer command polling can carry the same safe envelope;
- kept raw `payload_json` private and left existing command mappings with
  `transport: None` until NRCT-050 wires real URLs;
- updated Public OpenAPI and regenerated TypeScript/Kotlin SDK package entries;
- added protocol serialization and OpenAPI assertions for the safe envelope.

Gates:

```powershell
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Results:

- `nako-client-protocol public`: 12 passed.
- `nako-api public_openapi/sdk`: 17 passed, 44 skipped.
- format check passed.
- diff check passed.

## Gate Policy

Use focused gates while developing, then broaden only when a task crosses API,
SDK, runtime, or storage boundaries.

## Focused Gates

```powershell
python -m json.tool docs/workstreams/nako-renderer-cast-safe-transport/WORKSTREAM.json
git diff --check -- docs/workstreams/nako-renderer-cast-safe-transport docs/adr/0041-renderer-cast-safe-transport-tickets.md docs/adr/README.md docs/workstreams/README.md
cargo nextest run -p nako-server -E 'test(renderer) | test(playback)' --no-fail-fast
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(public_openapi) | test(admin_contract) | test(sdk)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Redaction Gates

For any Public/Admin DTO, generated SDK, diagnostic response, or command poll
response touched by this lane, tests should fail if output exposes:

- bearer token material;
- browser or renderer ticket values outside intended URL fields;
- raw Source Locators;
- local filesystem paths;
- Transcode Session IDs as credentials;
- raw `payload_json`;
- raw renderer capability JSON;
- owner principal internals.

## Closeout Gates

Before closing the lane:

```powershell
python -m json.tool docs/workstreams/nako-renderer-cast-safe-transport/WORKSTREAM.json
cargo nextest run -p nako-server -E 'test(renderer) | test(playback) | test(transcode)' --no-fail-fast
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(public_openapi) | test(admin_contract) | test(sdk)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```
