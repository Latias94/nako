# HLS Seek Restart Lifecycle - Design

Status: Active
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

## Boundary Direction

- `nako-transcode` owns HLS request-variant identity components, including the
  playback generation component.
- `nako-server` owns lifecycle policy: admission, reuse, cancellation,
  superseding, and playback-session linking.
- FFmpeg command planning receives an already-decided seek start; it does not
  own whether a request should supersede another request.
- Artifact serving reconstructs manifests from persisted request identity and
  only serves names that belong to that manifest.

## Risks

- Keyframe alignment and timestamp preservation are not solved by identity
  alone; later tasks must make FFmpeg seek flags explicit.
- Request identity must remain backward compatible for default playback.
- Finished sessions for the same seek generation may be reusable, but active
  sessions for another generation must not be confused with the current one.

## First Slice

The first slice adds the internal generation identity model and app-level
request field while preserving current public behavior. It proves:

- default playback request identity is unchanged;
- non-zero seek generation changes request identity and staging layout;
- request variant identity round-trips the generation component.

