# External Casting Adapter Boundary TODO

Status: Active
Last updated: 2026-05-27

## Task Ledger

### ECAB-010 - Open workstream and ADR

Status: Complete
Owner: current agent
Depends on: none

Scope:

- Create the workstream docs.
- Add ADR 0042 for sidecar renderer adapters.
- Link the lane from ADR/workstream indexes.

Validation:

```powershell
python -m json.tool docs/workstreams/external-casting-adapter-boundary/WORKSTREAM.json
git diff --check -- docs/workstreams/external-casting-adapter-boundary docs/adr/0042-sidecar-renderer-adapters-for-external-casting-protocols.md docs/adr/README.md docs/workstreams/README.md
```

### ECAB-020 - Characterize current external casting boundary

Status: Pending
Owner: current agent or worker
Depends on: ECAB-010

Scope:

- Prove Public renderer registration still rejects Chromecast, DLNA, and AirPlay
  targets.
- Prove Admin renderer diagnostics keep external protocol adapters planned
  while Nako remote-client cast-safe transport is ready.
- Prove diagnostics do not leak bearer tokens, renderer tickets, source
  locators, local paths, raw adapter payloads, or protocol-private addresses.

Likely files:

- `crates/nako-server/src/http/tests/renderer.rs`
- `crates/nako-api/src/admin/playback.rs`
- `crates/nako-server/src/http/admin.rs`

Validation:

```powershell
cargo nextest run -p nako-server -E 'test(renderer) | test(admin_v1_playback_renderers)' --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### ECAB-030 - Add host renderer adapter bridge contract

Status: Pending
Owner: current agent or worker
Depends on: ECAB-020

Scope:

- Add a host-side adapter contract for discovered external renderer targets and
  bounded command envelopes.
- Keep the contract host-owned; adapters submit device facts and receive
  command facts but never receive bearer tokens or source locators.
- Decide whether the first bridge transport uses Addon Task dispatch, Addon
  resource calls, or a dedicated adapter poll endpoint.

Likely files:

- `crates/nako-core/src/`
- `crates/nako-server/src/app/`
- `crates/nako-api/src/admin/playback.rs`
- `docs/adr/0042-sidecar-renderer-adapters-for-external-casting-protocols.md`

Validation:

```powershell
cargo nextest run -p nako-server renderer_adapter --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### ECAB-040 - Prove synthetic external adapter command flow

Status: Pending
Owner: current agent or worker
Depends on: ECAB-030

Scope:

- Add a fake Chromecast-like adapter in tests.
- Prove a discovered external renderer can become a Renderer Session controlled
  by the host.
- Prove play command dispatch receives a cast-safe media transport envelope and
  no forbidden internal credentials.
- Preserve denied-policy no-side-effect behavior.

Validation:

```powershell
cargo nextest run -p nako-server -E 'test(renderer_adapter) | test(renderer) | test(playback)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### ECAB-050 - Select first real protocol implementation

Status: Pending
Owner: current agent
Depends on: ECAB-040

Scope:

- Spike current Rust/sidecar library options for Chromecast and DLNA.
- Decide whether first real protocol work lands in `nako` or
  `nako-official-addons`.
- Record why the selected protocol goes first and what blocks the others.

Validation:

```powershell
git diff --check -- docs/workstreams/external-casting-adapter-boundary docs/adr
```

### ECAB-060 - Implement first real protocol adapter slice

Status: Pending
Owner: current agent or worker
Depends on: ECAB-050

Scope:

- Implement the first real protocol adapter slice chosen in ECAB-050.
- Keep host policy/ticket authority in Nako.
- Keep protocol discovery/control in the adapter boundary.

Validation:

```powershell
cargo nextest run -p nako-server -E 'test(renderer_adapter) | test(renderer)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

### ECAB-070 - Close or split follow-ons

Status: Pending
Owner: current agent
Depends on: ECAB-060

Scope:

- Run final gates.
- Update workstream evidence and handoff.
- Split Chromecast, DLNA, AirPlay, frontend casting picker, and network-access
  hardening if they outgrow this lane.

Validation:

```powershell
python -m json.tool docs/workstreams/external-casting-adapter-boundary/WORKSTREAM.json
cargo nextest run -p nako-server -E 'test(renderer_adapter) | test(renderer) | test(playback)' --no-fail-fast
cargo nextest run -p nako-api -E 'test(admin_contract) | test(public_openapi) | test(sdk)' --no-fail-fast
cargo fmt --all -- --check
git diff --check
```
