# 0040: Model Casting as Renderer Sessions and Protocol Adapters

## Status

Proposed.

## Context

Nako should support casting, but casting is not one protocol and not one
playback mode. A user may cast from the web UI to another authenticated Nako
client, to a Chromecast receiver, to a DLNA renderer, or eventually to an
AirPlay receiver. Those targets differ in discovery, authentication, media URL
support, control channel, queue semantics, and whether they can attach bearer
headers.

Jellyfin exposes the relevant product pressure through sessions with client
capabilities, supported remote-control commands, now-playing state,
controllable-session filtering, remote play/playstate/system commands,
SyncPlay groups, and configurable cast receiver applications. Nako should
support the same class of user workflows, but should not collapse all of that
into Playback Session or Transcode Session.

## Decision

Casting will be modeled as **Renderer Sessions** controlled through
protocol-specific **Renderer Adapters**.

The core relationship is:

```text
User command
  -> Renderer Session
  -> Playback Session
  -> optional Playback Transcode
  -> target transport URL/control message
```

Definitions:

- **Renderer Session** is the durable or semi-durable server record for a target
  device or target client that can receive playback/control commands.
- **Playback Session** remains the user/client playback attempt for a selected
  Media Source or Source Variant.
- **Transcode Session** remains an implementation artifact for remux/HLS.
- **Renderer Adapter** owns one protocol integration, such as Nako remote
  client, Chromecast, DLNA, or AirPlay.
- **Cast Ticket** is a short-lived playback transport credential for renderer
  devices that cannot attach normal bearer headers.

Implementation order:

1. Build the policy and target boundary first.
2. Implement **Nako-to-Nako cast** before external protocols. It uses
   authenticated Nako clients, the Public Client API, Playback Session
   heartbeat, and ordinary server-side permission checks.
3. Add Chromecast as a later adapter once cast-safe URL expiry, refresh,
   receiver application configuration, CORS/HTTPS/reverse-proxy behavior, and
   remote endpoint selection are accepted.
4. Add DLNA/AirPlay only after discovery/network trust, URL exposure, and
   limited-control semantics are explicit.

Admin diagnostics are part of the boundary. They expose redaction-safe renderer
session summaries and adapter readiness, but not owner principals,
capability JSON payloads, source locators, local paths, bearer tokens, cast
ticket material, or protocol-private network addresses.

External protocol readiness is not reported as a runtime failure while those
adapters are intentionally unimplemented. `nako_remote_client` can be ready
while Chromecast, DLNA, AirPlay, and non-direct cast-safe transport are
reported as planned adapter follow-ons.

Control commands should be typed and bounded:

```text
play item/source
pause
resume
seek
stop
set volume, if supported
show item/detail, if supported
```

The server must verify:

- the controlling user can see and play the item/source;
- the controlling user may control the target renderer;
- the target is still registered, reachable, and authorized for the command;
- any generated media URL is scoped to the selected source/session, target, and
  expiry window;
- remote-network use respects Remote Access Endpoint and trusted-proxy policy.

## Consequences

- Casting can evolve without making every Playback Session remotely
  controllable.
- Nako-to-Nako casting can ship first with fewer transport risks.
- Chromecast, DLNA, and AirPlay stay adapter concerns with shared policy,
  session, and ticket primitives.
- Cast-safe URLs are treated as secrets and do not expose raw Source Locators or
  local paths.
- Admin Web can show current renderer state and future adapter readiness without
  implying that planned protocols are broken runtime dependencies.
- SyncPlay/watch-party behavior remains a separate later lane; it can reuse
  renderer/session primitives when needed.

## Alternatives Considered

- **Implement Chromecast first:** deferred because receiver-app deployment,
  HTTPS, CORS, URL renewal, and no-bearer-header playback are harder than the
  host policy boundary itself.
- **Use Playback Session as the renderer record:** rejected because a single
  device session can browse, idle, receive commands, or control queue state
  before and after a specific media playback attempt.
- **Use Transcode Session IDs as cast URLs:** rejected because Transcode Session
  is an internal artifact and must not become a public transport identity.
- **Implement DLNA discovery inside core playback:** rejected because discovery
  and protocol quirks belong in adapters, not in the planner.

## Related Workstreams

- `docs/workstreams/casting-renderer-runtime/`
- `docs/workstreams/playback-policy-and-renderer-targets/`
- `docs/workstreams/network-access-boundary/`
- `docs/workstreams/browser-playback-auth-transport/`
