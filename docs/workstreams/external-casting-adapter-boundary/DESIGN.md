# External Casting Adapter Boundary Design

Status: Active
Last updated: 2026-05-27

## Problem

Nako can now cast to authenticated Nako remote clients with direct, remux, and
HLS media URLs protected by renderer transport tickets. External casting is a
different problem. Chromecast, DLNA, and AirPlay require local-network
discovery, protocol-specific control, and compatibility logic that should not
live in the Playback Planner or Public Client renderer routes.

## Target State

Nako supports external casting through protocol renderer adapters:

```text
Admin/User command
  -> host Renderer Session
  -> host Playback Session and planner
  -> host renderer cast-safe transport URL
  -> adapter command dispatch
  -> protocol receiver control
```

The host remains the authority for access, policy, sessions, tickets, and
redaction. The adapter remains the authority for discovery, protocol control,
receiver launch, and device compatibility.

## Relevant Authority

- ADR 0034: Addon Sidecar deployment and compatibility boundaries.
- ADR 0039: Playback policy and renderer target boundary.
- ADR 0040: Casting as Renderer Sessions and Protocol Adapters.
- ADR 0041: Renderer cast-safe transport tickets.
- ADR 0042: Sidecar renderer adapters for external casting protocols.

## Architecture Direction

Define a host-side Renderer Adapter Bridge before implementing a real protocol.
The bridge should be small:

```text
DiscoveredRendererTarget
  adapter_id
  stable_device_id
  target_kind
  display_name
  network_scope
  media_capabilities
  control_capabilities

AdapterCommandEnvelope
  renderer_session_id
  playback_session_id
  command
  source_id
  position_ms
  transport
```

The host may deliver command envelopes to an adapter through an Addon Task,
Addon resource call, or a later adapter-specific poll endpoint. That transport
choice is deliberately part of this workstream. The invariant is stricter than
the mechanism: adapters never receive bearer tokens or internal storage facts.

The first executable implementation should use a synthetic adapter/fake port.
That proves the host contract without depending on physical receivers,
multicast availability, platform APIs, or a third-party protocol crate.

## Protocol Order

Preferred order:

1. Host adapter contract and synthetic proof.
2. Chromecast spike and implementation decision.
3. DLNA/UPnP spike if Chromecast proves blocked or after Chromecast lands.
4. AirPlay only after network trust and platform constraints are explicit.

Chromecast is the likely first real protocol because it exercises receiver app
launch and cast-safe URLs without requiring the DLNA profile matrix. DLNA is a
close second if Rust Chromecast libraries or receiver-app requirements are not
good enough.

## In Scope

- host adapter bridge domain/API shape;
- adapter readiness and redaction diagnostics;
- synthetic adapter tests that prove policy, transport, and command boundaries;
- first protocol selection evidence;
- focused changes in `nako-server`, `nako-api`, and optionally
  `nako-official-addons` if the adapter boundary needs sidecar proof.

## Out Of Scope

- frontend casting picker UX;
- copying Jellyfin DLNA profiles or protocol code;
- broad Addon Manager lifecycle automation;
- remote internet casting before network-access policy explicitly allows it;
- SyncPlay, playlists/queues, subtitles, and multi-controller arbitration.

## Closeout Condition

This lane can close when:

- the host external adapter boundary is implemented or explicitly split;
- one synthetic adapter proof passes through policy, command, and cast-safe
  transport;
- the first real protocol implementation choice is recorded with evidence;
- Admin diagnostics remain redaction-safe;
- follow-on protocol work is either completed or split.
