# Nako Renderer Cast-Safe Transport Handoff

Status: Closed
Last updated: 2026-05-27

## Current State

The lane has implemented the Nako remote-client renderer transport primitive
needed before protocol-specific casting adapters.

The design decision is:

- browser playback tickets stay browser-only;
- renderer/cast-safe transport tickets are a separate credential type;
- renderer control routes remain bearer-authenticated;
- renderer media URLs can use cast-safe ticket auth;
- Chromecast, DLNA, and AirPlay are follow-on workstreams after this primitive
  exists.

`NRCT-050` is complete. Nako remote-client renderer play can now produce real
Direct, Remux, and HLS Playback Sessions with scoped command transport URLs.
Those URLs use renderer transport tickets, not browser playback tickets, and
HLS segment URLs are protected with the same renderer scope.

`NRCT-060` is complete. Admin renderer diagnostics now report
`nako_remote_client_cast_safe_transport` as ready, while Chromecast, DLNA, and
AirPlay remain planned protocol adapter follow-ons. ADR 0041 is accepted.

## Closeout

This lane is closed. The next execution lane is
`docs/workstreams/external-casting-adapter-boundary/`.

`NRCT-020` is complete. It locked the current gaps:

- renderer remux/HLS play commands are rejected before runtime records are
  created;
- `nako_remote_client + cast_ticket` registration is currently rejected;
- browser playback ticket responses do not carry Renderer Session, Playback
  Session, command, network scope, or cast-ticket transport scope.

`NRCT-030` added the renderer transport ticket service, `NRCT-040` added the
typed Public Client envelope, and `NRCT-050` wired both through renderer
registration, command queueing, command polling, direct/remux/HLS media routes,
and HLS segment protection.

`NRCT-070` ran closeout gates, updated final status/docs, and opened the first
protocol casting workstream for Chromecast/DLNA/AirPlay adapter design. Do not
add protocol discovery/control inside this closed lane.

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
