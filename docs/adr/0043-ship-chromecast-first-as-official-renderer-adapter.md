# 0043: Ship Chromecast First As An Official Renderer Adapter Sidecar

## Status

Accepted on 2026-05-27.

## Context

Nako now has a host-owned renderer adapter bridge and a synthetic external
adapter proof. The next decision is which real protocol should land first and
where the implementation should live.

The candidates are:

- Chromecast, which uses mDNS discovery and the Google Cast control protocol.
- DLNA/UPnP, which uses SSDP discovery, AVTransport/RenderingControl, and a
  large device-profile compatibility matrix.
- AirPlay, which has a different pairing, platform, and compatibility surface.

Jellyfin keeps Cast receiver application configuration separate from DLNA
device-profile/stream-building logic. Its DLNA model is profile-heavy: device
profiles describe direct-play, remux/transcode, codec, container, subtitle, and
bitrate constraints. That confirms Nako should not treat DLNA as the first
small protocol proof.

Current Rust options are also asymmetric. Chromecast has async sender crates
such as `oxicast` and `cast-sender`, while DLNA is mostly SSDP/UPnP building
blocks such as `ssdp-client` and `upnp-client`. A first slice should validate
the sidecar adapter boundary with the least protocol-profile breadth.

## Decision

The first real external casting protocol will be **Chromecast**, implemented as
an official renderer adapter sidecar in `nako-official-addons`.

Nako host responsibilities remain:

- addon protocol resource and scope shape;
- addon registration, grants, diagnostics, and routing;
- Renderer Session and Playback Session authority;
- Library Access and remote-control authorization;
- renderer cast-safe media transport;
- redaction of bearer tokens, Source Locators, local paths, raw payload JSON,
  and renderer ticket values.

The official Chromecast adapter responsibilities are:

- mDNS discovery and optional manually configured device connection;
- Google Cast receiver launch and media/control command translation;
- protocol retry/backoff and receiver compatibility handling;
- safe device facts and command results returned to the host.

The first implementation should use `oxicast` in the sidecar if it remains
build-compatible with the official addon workspace. `oxicast` is async,
Tokio-native, includes mDNS discovery behind a feature flag, can connect by IP,
launch the Default Media Receiver, load media URLs, and control playback. Its
optional local file server feature must stay disabled; Nako media transport
must come from host-issued cast-safe URLs.

DLNA follows after Chromecast and should get its own workstream because it
needs a Nako-native device-profile model before it can behave like a mature
media server feature. AirPlay remains deferred until pairing/auth and platform
constraints are explicit.

## Consequences

- Chromecast validates the real sidecar process boundary without importing LAN
  discovery or receiver protocol dependencies into `nako-server`.
- Nako can keep all playback policy and URL authority in one place.
- The first official adapter can be tested with manifest, payload, redaction,
  command-mapping, and optional live hardware smoke gates.
- DLNA is not blocked, but it is intentionally not forced through a
  Chromecast-shaped contract.
- The addon protocol needs a renderer-adapter resource/scope before the
  official sidecar can be registered as more than generic automation.

## Alternatives Considered

- **Implement DLNA first:** rejected for the first real slice. DLNA is
  important, but mature support needs device profiles and AVTransport/
  RenderingControl compatibility logic, not just SSDP discovery.
- **Embed `oxicast`, `cast-sender`, or `rust_cast` in `nako-server`:** rejected
  because local discovery, receiver lifecycle, and protocol dependency churn
  belong in sidecars.
- **Use the existing `automation` addon resource for renderer control:**
  rejected because renderer discovery/control is a durable product boundary and
  should not be hidden behind generic automation grants.
- **Start with AirPlay:** rejected until pairing, auth, platform discovery, and
  media transport constraints are better understood.

## Related Workstreams

- `docs/workstreams/external-casting-adapter-boundary/`
- `docs/workstreams/nako-renderer-cast-safe-transport/`
- `docs/workstreams/casting-renderer-runtime/`
- `F:/SourceCodes/Rust/nako-official-addons/docs/workstreams/official-chromecast-renderer-adapter/`
