# HLS Seek Restart Lifecycle - Design

Status: Completed
Last updated: 2026-05-29

## Problem

Current HLS request identity separates source revision, transcode profile,
adaptive ladder, and selected media renditions. It does not model playback
position. A future seek request would either collide with an existing HLS
session or reuse the wrong artifact directory.

The runtime also treats an in-flight matching request as a conflict. That is
acceptable for initial playback, but seek/restart needs a clearer lifecycle:
the host must know whether a request is the same generation, a new generation,
or a superseding generation that should cancel older work.

## Target State

```text
HlsSourceRequest
  -> HlsPlaybackGeneration
  -> HlsRequestVariantPlan identity
  -> TranscodeRequestIdentity
  -> isolated HLS staging layout
  -> session admission
  -> FFmpeg command with optional seek start
  -> manifest-backed artifact serving
```

The default generation is start position `0 ms`; it must not change existing
request keys or artifact paths. Non-default generations become part of the HLS
request variant identity.

The public source HLS playlist route accepts `start_position_ms` as the minimal
seek/restart surface. Client-player controls and UX remain outside this lane;
they can call the route with a start position and then continue through the
manifest-backed session segment URLs.

## Boundary Direction

- `nako-transcode` owns HLS request-variant identity components, including the
  playback generation component.
- `nako-server` owns lifecycle policy: admission, reuse, cancellation,
  superseding, and playback-session linking.
- FFmpeg command planning receives an already-decided seek start; it does not
  own whether a request should supersede another request.
- Artifact serving reconstructs manifests from persisted request identity and
  only serves names that belong to that manifest.

## Residual Risks

- Client-player seek controls are not implemented in this lane.
- Seek accuracy still depends on source keyframe distribution and FFmpeg input
  seek behavior. The command now makes timestamp and segment-boundary behavior
  explicit, but deeper source index/pre-roll optimization is a follow-on.
- Runtime resource scheduling for multiple users and high seek churn remains a
  separate playback scheduler lane.

## First Slice

The first slice adds the internal generation identity model and app-level
request field while preserving current public behavior. It proves:

- default playback request identity is unchanged;
- non-zero seek generation changes request identity and staging layout;
- request variant identity round-trips the generation component.

## Closeout

The lane shipped generation identity, restart admission, FFmpeg seek command
planning, and a public HLS playlist `start_position_ms` query. Default `0 ms`
requests preserve existing identity and command behavior.
