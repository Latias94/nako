# Nako Renderer Cast-Safe Transport TODO

Status: Active
Last updated: 2026-05-27

## Task Ledger

### NRCT-010 - Open workstream and ADR

Status: Complete
Owner: current agent
Depends on: none

Scope:

- Create the workstream docs.
- Add ADR 0041 for renderer cast-safe transport tickets.
- Update ADR/workstream indexes where they are stale.

Validation:

```powershell
python -m json.tool docs/workstreams/nako-renderer-cast-safe-transport/WORKSTREAM.json
git diff --check -- docs/workstreams/nako-renderer-cast-safe-transport docs/adr/0041-renderer-cast-safe-transport-tickets.md docs/adr/README.md docs/workstreams/README.md
```

Handoff:

- First executable code task is `NRCT-020`.

### NRCT-020 - Characterize current renderer transport gaps

Status: Complete
Owner: current agent or worker
Depends on: NRCT-010

Scope:

- Add tests proving renderer play currently rejects remux/HLS decisions.
- Add tests around current Nako renderer `transport_auth` behavior so the
  control-auth/media-auth split is explicit before changing it.
- Add tests proving browser playback tickets do not bind Renderer Session,
  Playback Session, or network scope, so they are not reused as cast tickets.
- Confirm denied policy paths create no playback session, transcode artifact,
  renderer command, or ticket.

Likely files:

- `crates/nako-server/src/http/tests/renderer.rs`
- `crates/nako-server/src/app/tests/playback.rs`
- `crates/nako-server/src/app/playback_ticket.rs`

Validation:

```powershell
cargo nextest run -p nako-server -E 'test(renderer) | test(playback_ticket) | test(playback)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### NRCT-030 - Add renderer transport ticket service

Status: Complete
Owner: current agent or worker
Depends on: NRCT-020

Scope:

- Add a renderer/cast-safe transport ticket service in `nako-server`.
- Bind issued tickets to principal, Renderer Session, Playback Session, Media
  Source, playback mode, network scope, and expiry.
- Validate expiry and scope mismatch failures.
- Keep debug output and errors redaction-safe.
- Keep the service interface storage-replaceable even if the first
  implementation is in-memory.

Likely files:

- `crates/nako-server/src/app/renderer_transport_ticket.rs`
- `crates/nako-server/src/app.rs`
- `crates/nako-server/src/app/composition.rs`
- focused tests under `crates/nako-server/src/app/`

Validation:

```powershell
cargo nextest run -p nako-server renderer_transport_ticket --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### NRCT-040 - Add safe renderer transport DTO contract

Status: Complete
Owner: current agent or worker
Depends on: NRCT-030

Scope:

- Add Public Client DTOs for a typed renderer media transport envelope.
- Expose the envelope through renderer play command response and command polling
  where the renderer needs it.
- Keep `payload_json` private and prevent token/source/path leakage in
  OpenAPI/SDK snapshots.
- Regenerate Rust/TypeScript SDK outputs if affected.

Likely files:

- `crates/nako-client-protocol/src/catalog.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-api/src/openapi.rs`
- `crates/nako-api/src/sdk.rs`
- generated SDK files

Validation:

```powershell
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### NRCT-050 - Enable Nako remote-client remux/HLS renderer playback

Status: In progress
Owner: current agent or worker
Depends on: NRCT-040

Scope:

- Allow `nako_remote_client + cast_ticket` media transport while keeping Public
  Client control routes bearer-authenticated.
- Update renderer playback orchestration so direct, remux, and HLS decisions
  can create Playback Sessions and safe transport envelopes.
- Reuse existing transcode/remux/HLS runtime paths; do not create placeholder
  playback sessions or expose Transcode Session IDs as credentials.
- Validate direct/remux/HLS ticketed media requests and HLS segment protection.
- Preserve no-side-effect behavior for denied policy and unsupported targets.

Likely files:

- `crates/nako-server/src/app/casting.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/renderer.rs`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-server/src/http/renderer.rs`
- `crates/nako-server/src/http/tests/renderer.rs`

Validation:

```powershell
cargo nextest run -p nako-server -E 'test(renderer) | test(playback) | test(transcode)' --no-fail-fast
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### NRCT-060 - Admin readiness, docs, and protocol follow-on split

Status: Pending
Owner: current agent or worker
Depends on: NRCT-050

Scope:

- Update Admin renderer diagnostics so Nako remote-client non-direct transport
  is ready only when the implementation and gates pass.
- Keep Chromecast, DLNA, and AirPlay as follow-on adapter workstreams.
- Update workstream evidence, milestones, and handoff.
- Confirm diagnostics do not leak tickets or raw command payloads.

Validation:

```powershell
cargo nextest run -p nako-server -E 'test(renderer) | test(admin)' --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### NRCT-070 - Close workstream

Status: Pending
Owner: current agent
Depends on: NRCT-060

Scope:

- Run final gates.
- Update `WORKSTREAM.json`, `EVIDENCE_AND_GATES.md`, `MILESTONES.md`, and
  `HANDOFF.md`.
- Add closeout notes and identify the first casting protocol workstream.

Validation:

```powershell
python -m json.tool docs/workstreams/nako-renderer-cast-safe-transport/WORKSTREAM.json
cargo nextest run -p nako-server -E 'test(renderer) | test(playback)' --no-fail-fast
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(public_openapi) | test(admin_contract) | test(sdk)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```
