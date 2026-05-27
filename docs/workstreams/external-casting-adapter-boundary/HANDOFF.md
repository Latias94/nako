# External Casting Adapter Boundary Handoff

Status: Active
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

## Next Task

Run `ECAB-050`.

`ECAB-050` should select the first real protocol implementation and repository
boundary. It should compare Chromecast and DLNA library/process options and
decide whether the real adapter belongs in `nako` or `nako-official-addons`.

## Important Files

- `docs/adr/0042-sidecar-renderer-adapters-for-external-casting-protocols.md`
- `docs/workstreams/external-casting-adapter-boundary/TODO.md`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/renderer.rs`
- `crates/nako-server/src/http/tests/renderer.rs`
- `crates/nako-api/src/admin/playback.rs`

## Cautions

- Do not move Chromecast/DLNA/AirPlay discovery into the Playback Planner.
- Do not give adapters bearer tokens, Source Locators, local paths, or
  Transcode Session IDs as credentials.
- Use a synthetic adapter proof before depending on physical receiver hardware.
