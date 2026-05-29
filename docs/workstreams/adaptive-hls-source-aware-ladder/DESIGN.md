# Adaptive HLS Source-Aware Ladder Runtime - Design

Status: Closed
Last updated: 2026-05-28

## Problem

Nako can now execute an adaptive HLS fMP4 request, but two shortcuts still make
the runtime too synthetic for real libraries:

- `HlsRendition::default_adaptive_ladder()` is a fixed 720p/480p ladder. It can
  request variants above the source resolution, ignore source bitrate, and
  ignore client max bitrate, width, or height constraints.
- Adaptive FFmpeg planning assumes every source has audio. It maps optional
  audio streams but still emits `var_stream_map` entries that reference audio
  for every variant, which is not a valid source shape for silent videos or
  video-only assets.

The server also reconstructs adaptive artifact manifests from the default
ladder, so any future dynamic ladder must be part of a durable identity or
manifest reconstruction boundary.

## Intent

Make adaptive fMP4 a typed, source-aware runtime shape. The desired state is a
deterministic ladder plan derived from selected media facts and playback
constraints, with request/session identity and artifact serving using the same
plan that FFmpeg executes.

## Refactor Brief

- **Intent:** remove fixed adaptive-ladder and always-audio assumptions before
  richer adaptive playback features multiply the runtime surface.
- **Scope:** `nako-transcode` HLS rendition planning, FFmpeg HLS command
  planning, profile/request identity helpers; `nako-playback` transcode
  requirement facts if needed; `nako-server` HLS staging, artifact
  reconstruction, session reuse, and redaction-covered playback flows.
- **Deletion plan:** retire direct server calls to
  `HlsRendition::default_adaptive_ladder()` for runtime adaptive fMP4 layout
  decisions; delete the audio-only adaptive stream-map assumption.
- **Boundary plan:** keep FFmpeg-specific argument assembly inside
  `nako-transcode`; keep source and client facts typed at the playback/server
  boundary; make the adaptive ladder plan stable enough for request identity
  and artifact reconstruction.
- **Testing plan:** add ladder-policy tests for source resolution, source
  bitrate, and client caps; add FFmpeg command-plan tests for audio and
  no-audio adaptive variants; add server tests for manifest reconstruction,
  reuse, playlist rewrite, and redaction-safe session output.
- **Risk plan:** keep MPEG-TS and fMP4 single-variant behavior unchanged; avoid
  copying reference-project implementation details; use deterministic fallback
  behavior when probe facts are missing; split any schema persistence change if
  the request-key plan is insufficient.
- **Workflow plan:** one durable workstream with three implementation slices:
  ladder policy and identity, no-audio command planning, then server runtime
  integration and closeout.

## Target Flow

```text
MediaProbeResult / selected streams
  -> PlaybackTargetProfile / TranscodeRequirement
  -> HlsAdaptiveLadderPlan
  -> TranscodeRequestIdentity
  -> HlsArtifactManifest
  -> HlsRequest / FfmpegCommandBuilder::hls
  -> TranscodeSession / HlsArtifactService
```

## Ladder Policy

The adaptive ladder should:

- never include a rendition larger than the known source resolution;
- respect known client max width, height, and bitrate constraints;
- cap target video bitrate by known source video bitrate when that prevents
  meaningless high-bitrate transcodes;
- produce at least one rendition for valid video sources, even when probe facts
  are incomplete;
- keep a stable versioned identity string so the same source/client policy
  reuses the same session and artifact service reconstructs the same allowed
  files.

The policy does not try to become a full Jellyfin device-profile model in this
lane. The output should be small, explainable, and deterministic.

## Audio Presence

Adaptive fMP4 command planning should carry whether the selected source has an
audio stream. For audio-bearing sources, FFmpeg may continue producing one
audio stream per variant. For video-only sources, the command plan must not map
audio streams and must emit video-only variant stream-map entries.

## Non-Goals

- Do not implement adaptive MPEG-TS.
- Do not implement alternate audio tracks, subtitle renditions, audio-only
  variants, LL-HLS, CMAF encryption, or DRM.
- Do not replace the FFmpeg CLI adapter with rsmpeg.
- Do not copy Jellyfin, FFmpeg, or rsmpeg source, schemas, tests, comments, or
  assets.

## Closeout Condition

This lane can close when:

- adaptive fMP4 ladder planning is source-aware and identity-stable;
- adaptive FFmpeg command planning supports both audio and no-audio sources;
- server staging, artifact reconstruction, playlist rewrite, reuse, and
  redaction-covered playback paths consume the same ladder plan;
- single-variant MPEG-TS/fMP4 behavior remains covered by focused gates;
- evidence is recorded and the workstream is marked closed.

## Closeout Summary

Closed on 2026-05-28 after the planned runtime deepening shipped:

- `HlsAdaptiveLadderPlan` now derives deterministic renditions from selected
  source video facts and `TranscodeOutputConstraints`, avoiding known upscales
  and capping variant bitrate by source/client facts.
- Adaptive request identity now carries a versioned `request_variant` key for
  the concrete ladder plan; session artifact reconstruction parses that key and
  rejects malformed adaptive ladder variants instead of silently serving a
  default shape.
- Adaptive FFmpeg command planning now emits audio-bearing or video-only maps,
  audio encoder arguments, and `var_stream_map` entries based on selected
  source audio presence.
- Server HLS staging and playback runtime consume the same ladder plan used by
  request identity and artifact serving.

Residual adaptive breadth remains split out: adaptive MPEG-TS, alternate audio
renditions, subtitle renditions, LL-HLS/CMAF/DRM, and a future second transcode
engine adapter evaluation.
