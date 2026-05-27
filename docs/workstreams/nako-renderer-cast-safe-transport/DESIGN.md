# Nako Renderer Cast-Safe Transport Design

Status: Active
Last updated: 2026-05-27

## Problem

Nako has a clean casting runtime for Nako-to-Nako direct play, but renderer play
commands currently stop when the Playback Planner chooses remux or HLS. That is
the correct temporary behavior, yet it blocks the next mature media-server
workflow: one authenticated client controls playback while another renderer
fetches policy-checked media through a URL that is safe for its media engine.

The architecture must also prepare for Chromecast, DLNA, and AirPlay. Those
protocols cannot be trusted to carry Nako bearer tokens in media byte requests,
so the host needs a reusable transport/ticket boundary before adapter-specific
discovery and receiver control is added.

## Product Pressure

Reference products make three things clear:

- session/control state is separate from the media URL consumed by a player or
  receiver;
- server-side policy and target capability decisions happen before transcode or
  remux work starts;
- cast and remote-player URLs must be expiring, scoped, and safe to show only
  to the selected renderer.

For Nako, that means Browser Playback Ticket, Renderer Session, Playback
Session, Transcode Session, and Renderer Cast-Safe Transport Ticket remain
separate concepts.

## Target State

Nako remote-client renderer playback supports direct, remux, and HLS decisions:

```text
controller Public Client request
  -> CastingAppService
  -> PlaybackAppService policy and planner
  -> Playback Session plus optional transcode artifact
  -> renderer cast-safe transport ticket
  -> typed Renderer Command transport envelope
  -> renderer media request validates ticket and scope
```

The control plane remains bearer-authenticated. The renderer must still
register, heartbeat, poll commands, and complete commands through Public Client
routes authenticated by its bearer credential.

The media plane is target-safe. It uses renderer-scoped URLs when the renderer
cannot attach bearer auth or when the selected playback shape needs a URL that
will be consumed outside the controller's browser/player context.

## Core Boundary

Introduce a server-owned renderer transport module in `nako-server`.

The first shape should be intentionally small:

```text
RendererTransportTicketCommand
  principal
  renderer_session_id
  playback_session_id
  source_id
  mode: direct | remux | hls
  network_scope
  now_ms

IssuedRendererTransport
  mode
  expires_at_ms
  content_type or playlist_type
  supports_range
  urls
```

The service may start as in-memory, matching browser ticket maturity, but the
interface must make persistence possible later. It must validate expiry and all
scope fields at use time.

## Transport Envelope

Renderer command DTOs need a safe media transport envelope. Do not expose raw
`payload_json` through Public Client DTOs. Prefer a typed field such as:

```text
RendererCommandTransportDto
  mode
  expires_at_ms
  content_type
  playlist_type
  supports_range
  stream_url
  playlist_url
```

The envelope may be returned on the controller's play response and on renderer
command polling. It should carry only target-consumable URLs and non-secret
metadata. The ticket value is present inside URLs and must be treated as a
secret by tests, logs, Admin diagnostics, and SDK snapshots.

## Registration Semantics

Today Nako remote renderer registration accepts bearer transport only. This was
good for the first direct-play command lane, but it conflates control auth and
media transport auth.

This lane should clarify the split:

- Public Client renderer routes always require bearer auth;
- `transport_auth` on the renderer target describes media byte transport;
- `nako_remote_client + bearer` remains valid for native clients that can fetch
  media with bearer auth;
- `nako_remote_client + cast_ticket` becomes valid for clients that use bearer
  control but need ticketed media URLs.

External protocols keep using adapter-owned registration/discovery later.

## Policy And Runtime Rules

- The Playback Planner remains the source of direct/remux/HLS decision and
  denial reasons.
- Playback permission policy must be checked before creating playback sessions,
  transcode artifacts, renderer commands, or tickets.
- Denied decisions must leave no runtime side effects.
- Renderer tickets must recheck current Library Access and playback policy at
  media request time when practical; otherwise the lifetime must be narrow and
  revocation handling must be explicit.
- HLS playlists and segments must not leak unauthenticated permanent segment
  URLs.
- Admin diagnostics may expose readiness and counts, but never ticket material,
  bearer tokens, local paths, source locators, command payload JSON, or raw
  capability payload JSON.

## Non-Goals

- No Chromecast receiver implementation in this lane.
- No DLNA or AirPlay discovery/control in this lane.
- No queue/SyncPlay/watch-party semantics.
- No frontend UI work.
- No broad rewrite of the transcode runtime unless the existing runtime cannot
  safely produce the needed remux/HLS transport artifacts.

## Open Design Notes

- Ticket persistence can remain in-memory for the first slice, but the service
  API must keep storage replaceable.
- Network scope should initially use the renderer target's known scope. Remote
  endpoint selection and trusted proxy policy can harden in the network access
  lane before external protocol exposure.
- If the existing Public Client command DTO cannot carry a typed envelope
  without awkward compatibility shims, prefer a breaking DTO change. Nako is not
  online and the user has approved fearless refactoring.
