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

Run or continue `NRCT-040`.

`NRCT-020` is complete. It locked the current gaps:

- renderer remux/HLS play commands are rejected before runtime records are
  created;
- `nako_remote_client + cast_ticket` registration is currently rejected;
- browser playback ticket responses do not carry Renderer Session, Playback
  Session, command, network scope, or cast-ticket transport scope.

`NRCT-030` added the renderer transport ticket service but intentionally did not
wire it into `NakoAppServices` yet. Wire it when `NRCT-040` or `NRCT-050`
needs the service to issue command transport URLs.

`NRCT-040` should add the typed Public Client renderer transport envelope and
keep raw `payload_json` private.

## Important Files

- `docs/adr/0041-renderer-cast-safe-transport-tickets.md`
- `docs/workstreams/nako-renderer-cast-safe-transport/TODO.md`
- `crates/nako-server/src/app/playback_ticket.rs`
- `crates/nako-server/src/app/renderer_transport_ticket.rs`
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
