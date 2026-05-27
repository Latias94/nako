# 0039: Keep Playback Policy and Renderer Targets Explicit

## Status

Proposed.

## Context

Nako now has durable Playback Sessions, a Playback Planner, short-lived browser
playback tickets, real User/Role/Library Access storage, and FFmpeg-backed
direct/remux/HLS execution. The next product pressure is not only "can this
file play?" It is "is this user allowed to play this way on this target?"

Jellyfin's mature surface shows the pressure clearly:

- user policy separates media playback, remux, audio transcode, video
  transcode, remote access, remote control, device allow-lists, active session
  limits, and SyncPlay;
- device profiles describe direct-play containers/codecs, transcode targets,
  bitrate limits, container/codec conditions, and subtitle delivery;
- playback-info responses bind candidate media sources, play session identity,
  selected method, transcode reasons, stream URLs, and error codes;
- session state tracks capabilities, now-playing data, supported remote-control
  commands, queues, and whether a session can be controlled remotely;
- cast receiver applications are configured separately from the playback
  planner itself.

Nako should absorb that feature pressure without copying Jellyfin's DLNA model
or letting HTTP routes, browser tickets, FFmpeg execution, and remote-control
protocols grow into one large playback decision surface.

## Decision

Nako will introduce an explicit **Playback Permission Policy** and
**Renderer Target** boundary before adding richer client playback, desktop
player, or casting behavior.

The policy layers are:

- **Library Access** remains the library visibility and basic browse/play/manage
  gate.
- **Playback Permission Policy** decides whether the resolved user/principal may
  use direct play, remux, audio transcode, video transcode, remote playback,
  remote control, casting, and future optimized-version workflows.
- **Playback Planner** consumes the effective permission policy and target
  capabilities. It returns allowed decisions and denial reasons; it does not
  query users, roles, or HTTP auth state by itself.
- **Renderer Target** describes where playback will happen. Browser, desktop
  native, mobile native, Nako remote client, Chromecast, DLNA renderer, and
  AirPlay are targets with different capability and transport requirements.
- **Transport Adapter** turns a selected playback plan into a safe delivery
  mechanism for that target: browser ticket URL, bearer-authenticated native
  stream, cast-safe URL, or protocol-specific control message.

Renderer targets are not media engines. They are capability and delivery
contexts. Transcode execution remains owned by the Playback Runtime and
Transcode Engine Adapter.

Initial domain shape should stay small:

```text
PlaybackPermissionPolicy
  allow_media_playback
  allow_direct_play
  allow_remux
  allow_audio_transcode
  allow_video_transcode
  allow_remote_playback
  allow_remote_control
  allow_cast
  max_streaming_bitrate
  max_remote_bitrate

PlaybackTarget
  kind: browser | native_desktop | native_mobile | nako_remote_client |
        chromecast | dlna_renderer | airplay
  network_scope: local | remote | unknown
  transport_auth: bearer | browser_ticket | cast_ticket | none
  capabilities: ClientPlaybackCapabilities
  control: RendererControlCapabilities
```

Public Client API may expose target/capability request fields and safe denial
reasons. Admin API may expose richer effective-policy diagnostics. Neither API
may expose raw Source Locators, local paths, FFmpeg command lines, token hashes,
or device secrets.

Crate ownership:

- `nako-core` owns shared policy records, target IDs, renderer-session IDs, and
  repository traits when persistence is needed.
- `nako-playback` owns pure planner inputs, target capability matching, and
  playback decision/denial vocabulary.
- `nako-server` owns effective policy resolution, app orchestration, ticket
  issuance, runtime settings, renderer adapter composition, and HTTP
  authorization.
- `nako-client-protocol` owns stable Public Client DTOs.
- `nako-api` owns Admin DTOs and generated contract surfaces.

## Consequences

- Per-user playback limits can be enforced before FFmpeg work is started.
- Direct, remux, audio-transcode, and video-transcode denial reasons become
  testable policy outcomes rather than route-level conditionals.
- Desktop and mobile clients can use the same planner with different targets
  instead of inheriting browser-only assumptions.
- Casting can be added as a renderer adapter later without making Chromecast,
  DLNA, or AirPlay concepts part of core playback planning.
- Admin diagnostics can explain effective permissions separately from runtime
  capability and FFmpeg hardware selection.
- Public Client contracts need forward-compatible capability and denial
  vocabulary.

## Alternatives Considered

- **Keep only Library Access = play:** rejected because it cannot express
  Jellyfin-class limits such as "browse and direct-play but do not transcode"
  or "local playback allowed, remote playback denied."
- **Copy Jellyfin UserPolicy and DeviceProfile wholesale:** rejected because
  Nako needs a smaller target-shaped model that works for browser, desktop,
  mobile, and future casting without inheriting DLNA-centric compatibility
  fields.
- **Make browser tickets the universal transport:** rejected because native
  clients and renderer devices have different auth and renewal mechanics.
- **Let renderer adapters decide transcode policy:** rejected because policy
  must remain server-owned, auditable, and consistent across clients.

## Related Workstreams

- `docs/workstreams/playback-policy-and-renderer-targets/`
- `docs/workstreams/casting-renderer-runtime/`
- `docs/workstreams/playback-transcode-policy-deepening/`
- `docs/workstreams/identity-and-library-access-contract/`
- `docs/workstreams/browser-playback-auth-transport/`
