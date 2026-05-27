# 0044: Profile-Driven Playback Capability Planner

## Status

Accepted.

## Context

Nako already has a **Playback Runtime**, durable Playback Sessions, policy-aware
renderer targets, remux/HLS execution, transcode runtime inventory, and typed
hardware acceleration policy records. The remaining architectural risk is that
playback compatibility is still mostly decided from a small client shape:
direct-play enabled, container list, video codec list, and audio codec list.

That shape is not deep enough for Jellyfin-class playback pressure:

- device and client profiles need container, codec, profile, level, bitrate,
  resolution, frame-rate, bit-depth, HDR, subtitle, and audio-channel
  conditions;
- direct play, remux, and **Playback Transcode** decisions need explainable
  reasons rather than a single enum value;
- hardware decode, filter, tone-map, subtitle burn-in, and encode choices need
  a higher-level pipeline requirement before FFmpeg command planning;
- **Client Applications** should be able to send capability profiles without
  inheriting Jellyfin's DLNA object model;
- Admin diagnostics should be able to explain why CPU, hardware decode, remux,
  or full transcode was selected.

`repo-ref/jellyfin` is reference material for behavior and feature pressure
only. Nako must not copy Jellyfin source, schemas, tests, or comments.

## Decision

Nako will replace the shallow client codec-list playback decision with a
profile-driven **Playback Capability Planner**.

The planner interface is playback-shaped:

```text
PlaybackPlanningRequest
  Media Source facts
  Media Technical Facts
  PlaybackTargetProfile
  EffectivePlaybackPolicy
  PlaybackSelectionContext

PlaybackDecision
  mode
  selected source
  execution plan
  decision report
  direct play / remux / transcode requirement
```

The profile model is not a direct copy of Jellyfin DLNA `DeviceProfile`.
Instead, Nako owns a smaller model:

- direct play capabilities;
- remux capabilities;
- transcode output capabilities;
- media compatibility conditions;
- subtitle delivery capabilities;
- audio/video output constraints;
- stable identity keys for cache and session reuse.

The decision report must carry typed reasons. Callers should not infer
compatibility by re-running planner logic.

`nako-playback` owns the pure planner and profile records. `nako-transcode`
owns FFmpeg, hardware inventory, acceleration policy, and command planning.
`nako-server` owns fact loading, access checks, policy resolution, persistence,
and HTTP adapters. Public Client API DTOs map into the Nako profile model; they
do not become the domain model.

The first implementation may keep output breadth narrow: direct play, remux,
and single-variant HLS H264/AAC. The important change is that the requirement
and reasons are deep enough to add hardware decode, HDR tone mapping, subtitle
burn-in, adaptive HLS, and optimized versions without route-level branching.

## Consequences

- Playback decisions become explainable and testable through one deep Module.
- Future hardware decode work can consume a transcode requirement instead of
  guessing from HTTP routes or FFmpeg command strings.
- Public Client capability contracts can evolve toward web, desktop, mobile,
  Chromecast, DLNA, and AirPlay without adopting Jellyfin DLNA structures.
- Compatibility tests must move from "codec list matches" toward condition and
  reason matrix coverage.
- Existing shallow planner records may be deleted or converted into adapters
  during the refactor because Nako is not yet a compatibility-constrained
  production system.

## Alternatives Considered

- **Keep `ClientPlaybackCapabilities` as container/codecs only:** rejected
  because it cannot express Jellyfin-class compatibility reasons or future
  subtitle/HDR/hardware decisions.
- **Copy Jellyfin `DeviceProfile`:** rejected because Nako's clients are not
  DLNA-first and the repo reference license rules forbid copying.
- **Put capability checks in `nako-transcode`:** rejected because playback
  compatibility is user/client/source policy, while transcode is an execution
  adapter and hardware/runtime boundary.
- **Jump straight to richer FFmpeg hardware parameters:** rejected because it
  would deepen the wrong Module. FFmpeg command planning should execute a
  pipeline requirement produced by playback policy.

## Related Workstreams

- `docs/workstreams/playback-capability-profile-planner/`
- `docs/workstreams/playback-transcode-policy-deepening/`
- `docs/workstreams/playback-policy-and-renderer-targets/`
- `docs/workstreams/playback-transcode-ops-hardening/`

