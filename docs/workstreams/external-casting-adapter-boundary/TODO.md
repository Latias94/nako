# External Casting Adapter Boundary TODO

Status: Complete
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

Status: Complete
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

Evidence:

- Public renderer registration rejects Chromecast, DLNA, and AirPlay targets.
- The rejection explicitly points callers to the adapter boundary.
- Admin renderer diagnostics keep external protocol adapters planned while Nako
  remote-client cast-safe transport remains ready.
- Admin diagnostics do not leak renderer ticket query values or ticket prefixes.

### ECAB-030 - Add host renderer adapter bridge contract

Status: Complete
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

Evidence:

- Added `app::renderer_adapter` with a host-owned bridge service.
- External adapter targets are limited to Chromecast, DLNA, and AirPlay.
- Adapter targets are local-network scoped until remote casting policy is
  accepted.
- Playback target projection always uses cast-ticket transport.
- Adapter command envelopes are bounded to registered targets and redaction
  tested.

### ECAB-040 - Prove synthetic external adapter command flow

Status: Complete
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

Evidence:

- App composition exposes the renderer adapter bridge as a host service.
- Adapter-published external targets can become host-owned Renderer Sessions
  without using Public renderer registration.
- Synthetic Chromecast-like play command uses the existing renderer playback
  pipeline and returns cast-safe transport URLs.
- Synthetic adapter command envelopes are built from internal transport plans
  without bearer tokens, Source Locators, local paths, raw payload JSON, or
  renderer ticket values.
- Remote-control policy denial creates no Playback Session, Renderer Command,
  or Transcode Session side effects.

### ECAB-050 - Select first real protocol implementation

Status: Complete
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

Evidence:

- Added `PROTOCOL_SELECTION.md`.
- Accepted ADR 0042 and added ADR 0043.
- Selected Chromecast as the first real protocol.
- Selected `nako-official-addons` as the implementation repository for the
  protocol sidecar.
- Selected `oxicast` as the preferred first sidecar dependency, with
  `cast-sender` as the fallback.
- Deferred DLNA until a renderer device-profile workstream exists.

### ECAB-060 - Implement first real protocol adapter slice

Status: Complete
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

Evidence:

- Nako addon protocol now declares the `renderer_adapter` resource, read/control
  scopes, and typed readiness/discovery/command payloads.
- The official catalog now includes
  `nako.official.chromecast-renderer` with hosted diagnostics, resource schema,
  runtime binary, and container image facts.
- `nako-official-addons` commit `18d3df0` adds
  `nako-chromecast-renderer`, package files, smoke script, and its workstream.
- The official sidecar validates resource envelopes, publishes manual
  Chromecast targets, builds redaction-safe Chromecast command plans, links
  `oxicast`, and keeps live LAN discovery/control behind explicit flags.

### ECAB-070 - Close or split follow-ons

Status: Complete
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

Evidence:

- Added `FOLLOW_ONS.md` for live Chromecast hardening, DLNA renderer profiles,
  AirPlay feasibility, frontend casting picker, and network trust policy.
- Kept physical receiver smoke out of required CI gates.
- Closeout gates are recorded in `EVIDENCE_AND_GATES.md`.
