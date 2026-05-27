# Nako Renderer Cast-Safe Transport Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The lane has been opened to implement Nako remote-client non-direct renderer
transport before protocol-specific casting adapters.

The design decision is:

- browser playback tickets stay browser-only;
- renderer/cast-safe transport tickets are a separate credential type;
- renderer control routes remain bearer-authenticated;
- renderer media URLs can use cast-safe ticket auth;
- Chromecast, DLNA, and AirPlay are follow-on workstreams after this primitive
  exists.

## Next Task

Run or continue `NRCT-020`.

Start by adding characterization tests for:

- current direct-only renderer playback behavior;
- current Nako renderer `transport_auth` registration semantics;
- browser tickets lacking renderer/playback-session/network-scope binding.

## Important Files

- `docs/adr/0041-renderer-cast-safe-transport-tickets.md`
- `docs/workstreams/nako-renderer-cast-safe-transport/TODO.md`
- `crates/nako-server/src/app/playback_ticket.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/casting.rs`
- `crates/nako-server/src/app/renderer.rs`
- `crates/nako-server/src/http/playback.rs`
- `crates/nako-server/src/http/renderer.rs`
- `crates/nako-server/src/http/tests/renderer.rs`

## Cautions

- Do not reuse browser playback tickets as renderer/cast tickets.
- Do not expose `payload_json` through Public Client DTOs.
- Do not use Transcode Session IDs as public media credentials.
- Keep Admin diagnostics redaction-safe.
- Do not start Chromecast/DLNA/AirPlay implementation in this lane.
