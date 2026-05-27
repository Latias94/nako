# Casting Renderer Runtime

Status: Planned
Last updated: 2026-05-27

## Why This Lane Exists

Nako should support casting, but casting should not be bolted onto browser
playback URLs or Transcode Session IDs. Mature media servers make casting feel
like "play this on that device," but the implementation needs separate
concerns:

- who is allowed to control the target;
- what the target can play;
- how the target receives media URLs or commands;
- whether playback is local or remote;
- how progress, pause/seek/stop, and current connection state are tracked.

This lane starts only after `playback-policy-and-renderer-targets` establishes
effective playback policy and renderer target vocabulary.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0036-short-lived-browser-playback-tickets.md`
- `docs/adr/0039-playback-policy-and-renderer-target-boundary.md`
- `docs/adr/0040-casting-as-renderer-session-adapter.md`
- `docs/workstreams/playback-policy-and-renderer-targets/`
- `docs/workstreams/network-access-boundary/`
- `docs/workstreams/browser-playback-auth-transport/`
- `docs/workstreams/playback-transcode-policy-deepening/`

## Jellyfin-Class Feature Pressure

The reference behavior pressure from Jellyfin:

| Jellyfin pressure | Nako casting boundary |
| --- | --- |
| Sessions expose capabilities, supported commands, now-playing state, queue, and remote-control support. | Renderer Session read model and control capability records. |
| Session routes send browse, play, playstate, system, message, and command payloads to target sessions. | Typed renderer commands with permission checks and adapter dispatch. |
| User policy controls remote control and shared device control. | Effective playback/control policy from the previous lane. |
| PlaybackInfo and SessionInfo are separate surfaces. | Playback Session remains distinct from Renderer Session. |
| Cast receiver applications are configured separately. | Protocol adapter configuration, not planner state. |
| SyncPlay is a separate manager and group model. | Watch-party/SyncPlay stays out of first casting runtime. |

## Problem

Without this lane, casting would likely grow in the wrong place:

- browser tickets would be stretched into long-lived cast URLs;
- Transcode Session IDs might become public transport identities;
- playback routes would need protocol-specific branches for Chromecast or DLNA;
- device discovery and control commands would leak into the Playback Planner;
- users could control devices without a durable policy/audit boundary;
- external renderers that cannot attach bearer headers would require ad hoc
  exceptions.

## Target State

When this lane closes:

- Nako has Renderer Session records for controllable target clients/devices.
- Nako has a renderer-control app service with typed commands: play, pause,
  resume, seek, stop, and optional browse/show-item.
- Nako-to-Nako casting works as the first implementation: one authenticated
  Nako client can register as a target and another authorized client can send a
  play command.
- Playback Session remains the selected media attempt; Renderer Session is the
  target/control session; Transcode Session remains internal.
- Cast-safe media transport uses scoped ticket/session URLs when bearer headers
  cannot be used.
- Chromecast/DLNA/AirPlay are left as protocol adapters with explicit follow-on
  tasks unless this lane reaches them after the Nako-to-Nako proof.
- Admin diagnostics can show redaction-safe active renderer sessions,
  controllability, and adapter readiness.

## In Scope

- Workstream docs and ADR 0040.
- Renderer Session domain records and repository traits if not already shipped
  by the previous lane.
- Renderer registration/heartbeat read model for Nako clients.
- Typed renderer commands and command dispatch.
- Nako-to-Nako cast proof through Public Client API and server-side policy.
- Cast-safe ticket shape only if a target cannot use bearer auth.
- Admin diagnostics/readiness for renderer sessions and adapter state.
- Follow-on docs for Chromecast, DLNA, and AirPlay adapters.

## Out Of Scope

- Full Chromecast implementation in the first slice.
- DLNA/UPnP discovery in the first slice.
- AirPlay implementation in the first slice.
- SyncPlay/watch-party groups.
- Frontend UI.
- Native mobile implementation.
- Built-in NAT traversal or relay.
- Public sharing links.
- Recommendation or queue intelligence.
- Copying reference project code.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Nako-to-Nako cast should ship before Chromecast/DLNA/AirPlay. | High | Auth, capability negotiation, heartbeat, and progress can reuse Nako Public Client primitives. | If external casting becomes urgent, keep the same Renderer Adapter interface and implement one protocol adapter earlier. |
| Renderer Session must be separate from Playback Session. | High | A target can exist, idle, browse, or be controllable before a media playback attempt. | If this proves too heavy, keep Renderer Session process-local first but preserve ID/vocabulary for later persistence. |
| Cast URLs need a separate ticket lifecycle from browser tickets. | Medium | Chromecast/DLNA often cannot attach bearer headers and may need longer renewal behavior. | If Nako-to-Nako only uses bearer auth, cast-ticket implementation can be deferred until first external adapter. |
| Remote access policy matters for casting. | High | External renderers may fetch URLs through a reverse proxy or remote endpoint. | If first implementation is LAN-only, record that network scope explicitly and block remote targets. |
| SyncPlay is separate. | High | Jellyfin keeps group synchronization distinct from ordinary remote control. | If watch-party is requested, split a `syncplay-session-runtime` lane. |

## Architecture Direction

### Runtime Model

```text
RendererSession
  id
  owner_principal_id
  target_kind
  display_name
  network_scope
  capabilities
  supported_commands
  state
  last_seen_at

RendererCommand
  id
  controlling_principal_id
  renderer_session_id
  command_kind
  playback_session_id?
  item_id?
  source_id?
  position_ms?
  created_at
```

The first implementation may keep command delivery process-local if that is the
fastest truthful proof, but the Interface should not assume one server process
forever.

### Adapter Boundary

```text
RendererAdapter
  register/refresh target
  send command
  inspect readiness
```

Adapters:

- `nako_remote_client`: first implementation, authenticated Nako client.
- `chromecast`: later, receiver application plus cast-safe URL behavior.
- `dlna_renderer`: later, discovery/control adapter with limited command set.
- `airplay`: later, protocol-specific adapter after network/security review.

### Security Boundary

Every command must check:

- controlling user is authenticated;
- controlling user has playback/control permission for the target;
- controlling user has Library Access and playback policy for the item/source;
- target supports the command;
- network scope and Remote Access Endpoint policy allow the transport;
- ticket/URL expiry is scoped and redaction-safe.

### Relationship To Playback Runtime

Renderer runtime may request Playback Session creation, but it does not become
the Playback Planner or Transcode Engine. It calls the same policy-aware
Playback App Service that browser/native playback uses.

## Closeout Condition

This lane can close when:

- ADR 0040 and workstream docs match shipped behavior;
- Renderer Session and command boundaries exist;
- Nako-to-Nako cast is implemented or deliberately split after a smaller proof;
- casting does not expose raw Source Locators, local paths, bearer tokens, or
  Transcode Session IDs;
- Admin diagnostics and Public Client DTOs expose only safe renderer/session
  facts;
- Chromecast/DLNA/AirPlay follow-ons are explicit with adapter boundaries.
