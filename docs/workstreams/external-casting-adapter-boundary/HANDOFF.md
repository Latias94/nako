# External Casting Adapter Boundary Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

The workstream is open. ADR 0042 proposes sidecar renderer adapters for
external casting protocols. Nako remote-client cast-safe transport is already
complete in `docs/workstreams/nako-renderer-cast-safe-transport/`.

`ECAB-020` is complete. The current external boundary is now locked by tests:
Public renderer registration rejects Chromecast, DLNA, and AirPlay targets;
Admin diagnostics keep those adapters planned; and diagnostics do not leak
renderer ticket values.

`ECAB-030` is complete. `app::renderer_adapter` now owns the first host-side
adapter bridge contract for external renderer targets and bounded command
envelopes.

`ECAB-040` is complete. A synthetic Chromecast-like adapter target can become a
host-owned Renderer Session, use the existing play command pipeline, receive
cast-safe transport, and preserve denied-policy no-side-effect behavior.

`ECAB-050` is complete. Chromecast is selected as the first real protocol,
implemented as an official sidecar in `nako-official-addons`. `oxicast` is the
preferred first dependency, `cast-sender` is the fallback, and DLNA is deferred
until a Nako renderer device-profile workstream exists.

`ECAB-060` is complete. Nako now has the typed addon protocol surface and
official catalog descriptor for renderer adapters, and `nako-official-addons`
commit `18d3df0` adds the `nako-chromecast-renderer` sidecar.

`ECAB-070` is complete. Remaining mature casting work is split into
`FOLLOW_ONS.md`.

## Next Task

Recommended next lane: live Chromecast hardening only after a local receiver can
be used for manual smoke, or DLNA renderer-profile design if the priority is
Jellyfin-like broad device compatibility.

## Important Files

- `docs/adr/0042-sidecar-renderer-adapters-for-external-casting-protocols.md`
- `docs/adr/0043-ship-chromecast-first-as-official-renderer-adapter.md`
- `docs/workstreams/external-casting-adapter-boundary/PROTOCOL_SELECTION.md`
- `docs/workstreams/external-casting-adapter-boundary/FOLLOW_ONS.md`
- `docs/workstreams/external-casting-adapter-boundary/TODO.md`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/renderer.rs`
- `crates/nako-server/src/http/tests/renderer.rs`
- `crates/nako-api/src/admin/playback.rs`
- `F:/SourceCodes/Rust/nako-official-addons/crates/nako-chromecast-renderer`

## Cautions

- Do not move Chromecast/DLNA/AirPlay discovery into the Playback Planner.
- Do not give adapters bearer tokens, Source Locators, local paths, or
  Transcode Session IDs as credentials.
- Use a synthetic adapter proof before depending on physical receiver hardware.
- Keep live Chromecast hardware smoke optional unless CI has a controlled
  receiver fixture.
