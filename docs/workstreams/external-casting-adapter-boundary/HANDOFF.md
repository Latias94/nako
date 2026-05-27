# External Casting Adapter Boundary Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The workstream is open. ADR 0042 proposes sidecar renderer adapters for
external casting protocols. Nako remote-client cast-safe transport is already
complete in `docs/workstreams/nako-renderer-cast-safe-transport/`.

## Next Task

Run `ECAB-020`.

`ECAB-020` should characterize the current boundary before adding any adapter
implementation:

- Public renderer registration still rejects Chromecast, DLNA, and AirPlay;
- Admin diagnostics keep those external protocol adapters planned;
- Nako remote-client cast-safe transport remains ready;
- diagnostics do not leak credentials, source locators, local paths, or raw
  payloads.

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
