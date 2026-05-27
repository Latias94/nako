# 0042: Use Sidecar Renderer Adapters For External Casting Protocols

## Status

Accepted on 2026-05-27.

## Context

Nako remote-client casting now has host-owned Renderer Sessions, Playback
Sessions, renderer-scoped transport tickets, and typed command transport
envelopes. The remaining casting protocols are external to the authenticated
Nako client model:

- Chromecast needs device discovery, receiver application control, and a
  protocol bridge that cannot carry Nako bearer tokens in media requests.
- DLNA/UPnP needs local-network discovery, device profiles, and noisy multicast
  behavior that should not become part of playback planning.
- AirPlay needs platform/protocol-specific discovery and control with a
  different trust and compatibility surface.

Nako already has an Addon Sidecar model for extension code with different
dependency, network, credential, and lifecycle profiles. ADR 0034 explicitly
calls out DLNA/UPnP compatibility as a likely sidecar-shaped capability.

## Decision

External casting protocols will be implemented as **sidecar renderer
adapters**, not as playback-planner logic and not as Public Client renderer
clients.

The Nako host owns:

- Library Access and playback policy checks;
- Renderer Session and Playback Session records;
- command authorization and lifecycle state;
- renderer cast-safe transport ticket issue/validation;
- Admin readiness and redaction;
- the stable adapter contract.

The protocol adapter owns:

- device discovery and reachability checks;
- protocol-specific capability mapping;
- receiver application launch or device command translation;
- protocol retry/backoff and compatibility quirks;
- local-network behavior such as multicast or platform discovery.

Adapters may receive target-safe media URLs and bounded command facts. They
must not receive bearer tokens, raw Source Locators, local filesystem paths,
raw command payload JSON, or Transcode Session IDs as credentials.

Nako remote clients remain on the Public Client renderer session path. External
protocol renderers use a host-owned adapter bridge that can later be backed by
official sidecars.

## Consequences

- Chromecast, DLNA, and AirPlay can reuse the cast-safe transport primitive
  without copying media authorization into each adapter.
- LAN discovery dependencies and protocol-specific libraries do not become
  mandatory server-core dependencies.
- Official adapters can live in `nako-official-addons` or another sidecar
  package while Nako core keeps the durable authority model.
- Admin diagnostics can report adapter readiness separately from Nako
  remote-client readiness.
- The first implementation should prove the host/adapter contract with a
  synthetic adapter before committing to a real protocol crate or process.

## Alternatives Considered

- **Embed Chromecast/DLNA/AirPlay libraries directly in `nako-server`:**
  rejected because discovery, network permissions, receiver quirks, and
  dependency churn do not belong in the playback planner or core HTTP server.
- **Treat external receivers as Public Client renderers:** rejected because
  external devices cannot authenticate, heartbeat, poll commands, or complete
  commands like a Nako client.
- **Let each protocol mint its own media URLs:** rejected because Nako host
  must remain the authority for playback policy, source access, expiry,
  network scope, and redaction.

## Related Workstreams

- `docs/workstreams/external-casting-adapter-boundary/`
- `docs/workstreams/nako-renderer-cast-safe-transport/`
- `docs/workstreams/casting-renderer-runtime/`
- `docs/workstreams/addon-ecosystem-foundation/`
- `docs/workstreams/network-access-boundary/`
