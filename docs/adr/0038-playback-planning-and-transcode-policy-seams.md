# 0038: Deepen Playback Planning and Transcode Policy Seams

## Status

Proposed.

## Context

Nako now has durable Playback Sessions, optional Transcode Sessions, browser
playback tickets, FFmpeg-backed remux/HLS execution, runtime diagnostics, and
Admin/Public Client playback routes. The next risk is that playback decisions
and FFmpeg/runtime choices remain spread across HTTP handlers, app playback
modules, `nako-transcode` command building, and Admin diagnostics.

Jellyfin's mature playback surface shows the pressure Nako must be ready for:

- device profiles that describe direct-play containers/codecs, transcoding
  targets, bitrate limits, subtitle delivery, and codec/container conditions;
- playback-info responses that explain direct play, remux, transcode, stream
  URLs, selected streams, and transcode reasons;
- encoding options for hardware acceleration, encoder paths, tonemapping,
  throttling, segment deletion, subtitles, and per-codec hardware decode;
- transcode job management with play-session pings, progress, cleanup,
  throttling, cancellation, and safe current-session reporting;
- user policy that can allow or deny remux/audio transcode/video transcode.

Nako should learn from those feature pressures without copying Jellyfin code or
making Jellyfin's DLNA model the Nako domain model.

## Decision

Nako will introduce explicit playback planning and transcode policy seams before
expanding hardware acceleration, client capability negotiation, adaptive HLS,
optimized versions, or desktop-native playback.

The target architecture is:

- **Playback Session** remains the durable user/client playback attempt.
- **Playback Planner** decides direct play, remux, HLS transcode, or future
  optimized/remote playback from principal, Library Access, Media Source facts,
  Client Playback Capabilities, Library Playback Policy, and Server Runtime
  Capabilities.
- **Transcode Policy** turns a transcode requirement into a typed acceleration
  and output plan. It must model decode, filter/scale/tonemap, encode, subtitle
  handling, bitrate, and fallback independently; a single
  `hardware_acceleration: bool` is not enough.
- **Playback Runtime Inventory** records redaction-safe FFmpeg and hardware
  capability snapshots used by planner and Admin diagnostics.
- **Transcode Engine Adapter** executes an already-planned artifact. The first
  Adapter remains FFmpeg CLI. Future adapters may include remote transcode
  workers or precomputed optimized versions, but they must satisfy the same
  engine interface.
- **Admin Runtime Settings** owns mutable operator policy. Public Client DTOs
  only receive safe playback decisions, reasons, URLs/tickets, and session
  state.

Crate ownership stays conservative:

- `nako-core` owns pure records, IDs, policy enums, and repository traits when
  shared across crates.
- `nako-transcode` owns FFmpeg command planning, capability probing, and the
  FFmpeg engine Adapter.
- `nako-streaming` owns byte/HLS/remux response mechanics.
- `nako-server` owns orchestration, persistence, access checks, planner
  application, runtime settings, and HTTP boundaries.
- `nako-api` owns Admin DTOs.
- `nako-client-protocol` owns Public Client playback DTOs only.

Nako will not directly import Jellyfin code, schemas, comments, tests, or assets
from `repo-ref/`. `repo-ref/oximedia` and `repo-ref/libmedia` remain reference
material for media pipeline layering and client capability vocabulary; they are
not server dependencies for this decision.

## Consequences

- Mature playback features have named seams before they are implemented.
- Direct play, remux, and HLS can share one planning model while preserving
  distinct execution paths.
- Hardware acceleration becomes explainable: selected stage, selected device,
  fallback, and failure reason can be recorded without leaking raw FFmpeg
  commands or local paths.
- Admin diagnostics and Public Client playback decisions stay separate.
- Desktop-native playback can consume Playback Session and Client Playback
  Capability contracts without inheriting browser ticket mechanics.
- More pure policy tests are required before route cleanup.
- Some existing route-level playback decisions and `nako-transcode` request
  shapes will become adapters around deeper planner/policy records.

## Alternatives Considered

- **Keep route-owned playback decisions:** rejected because each new feature
  would duplicate direct/remux/transcode capability checks and hardware fallback
  logic.
- **Make FFmpeg command building the policy:** rejected because command strings
  are too low-level to explain user-facing decisions, Admin settings, fallback,
  or future non-FFmpeg adapters.
- **Adopt Jellyfin's DLNA `DeviceProfile` model directly:** rejected because
  Nako needs a smaller, explicit Public Client capability model for web,
  desktop, and mobile rather than a copied DLNA-centric domain.
- **Introduce a new playback crate immediately:** deferred. Start with
  `nako-core` records plus `nako-server` deep modules, then extract only if
  multiple real adapters/callers create reuse pressure.
- **Adopt `oximedia` or `libmedia` as server dependencies:** deferred. They are
  useful references, but Nako's server needs a control plane and engine Adapter
  seam more than a new embedded media stack.

## Related Workstreams

- `docs/workstreams/playback-transcode-policy-deepening/`
- `docs/workstreams/backend-media-product-deepening/`
- `docs/workstreams/browser-playback-auth-transport/`
- `docs/workstreams/playback-transcode-ops-hardening/`
- `docs/workstreams/transcode-runtime/`

