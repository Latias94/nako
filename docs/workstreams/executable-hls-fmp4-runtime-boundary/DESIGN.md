# Executable HLS fMP4 Runtime Boundary - Design

Status: Completed
Last updated: 2026-05-28

## Problem

The playback media maturity refactor added `HlsOutputRequirement` to planning,
but the executable HLS runtime still assumes MPEG-TS everywhere:

- `HlsStagingPolicy` always emits `segment_%05d.ts`.
- `HlsRequest` does not carry output requirement intent.
- `FfmpegCommandBuilder::hls` always builds the same HLS muxer arguments.
- HLS segment serving always reports `video/mp2t`.
- stale segment cleanup only considers `.ts` files.

That means Public Client fMP4 capability input can influence planning identity,
but runtime execution silently collapses back to MPEG-TS. Nako should remove
that shallow output assumption before adaptive HLS ladders add more branching.

## Intent

Make HLS output shape an explicit runtime boundary. The first executable slice
adds fMP4 single-variant output without implementing full adaptive bitrate
ladders.

## Refactor Brief

- **Intent:** delete the implicit `.ts`/MPEG-TS-only HLS output assumption and
  make output shape travel with the transcode request.
- **Scope:** `nako-transcode` HLS request/FFmpeg muxer planning,
  `nako-server` HLS staging/artifact serving, `nako-playback` requirement
  consumption, generated docs/workstream evidence.
- **Deletion plan:** remove hard-coded `segment_%05d.ts` and `video/mp2t`
  assumptions from the HLS runtime path where output requirement is available.
- **Boundary plan:** `HlsOutputRequirement` owns segment container and variant
  policy; staging policy owns layout; FFmpeg builder owns muxer flags; artifact
  service owns segment content type.
- **Testing plan:** command-plan tests for MPEG-TS and fMP4, staging layout
  tests, server HLS route tests for fMP4 segment content type, and focused
  package gates.
- **Risk plan:** keep adaptive variants non-executable in this lane; reject or
  downgrade unsupported runtime shapes explicitly rather than silently claiming
  full ABR support.
- **Workflow plan:** durable workstream with one bounded runtime slice, then
  review, verification, closeout, and user-confirmed commit.

## Scope

- `crates/nako-transcode`: `HlsRequest`, command identity/validation, FFmpeg
  HLS muxer args.
- `crates/nako-server`: HLS staging policy, HLS app service, artifact service,
  playlist/segment tests.
- `crates/nako-playback`: use `TranscodeRequirement.hls_output` when creating
  runtime requests.
- `docs/workstreams`: task ledger, evidence, and closeout.

## Non-Goals

- Do not implement adaptive bitrate ladder generation.
- Do not implement multi-variant master playlists.
- Do not add CMAF encryption, LL-HLS, or DRM.
- Do not replace FFmpeg CLI with rsmpeg.
- Do not copy Jellyfin source, schemas, tests, or comments.

## Target Flow

```text
ClientPlaybackCapabilities
  -> PlaybackTargetProfile.hls_output
  -> TranscodeRequirement.hls_output
  -> HlsSourceRequest / HlsOutputLayout
  -> HlsRequest
  -> FfmpegCommandBuilder::hls
  -> HlsArtifactService::plan_segment
```

MPEG-TS remains the default. fMP4 should become an explicitly selected
single-variant output shape with `.m4s` media segments and an init segment.

## Closeout Condition

This lane can close when:

- HLS runtime requests carry `HlsOutputRequirement`.
- fMP4 and MPEG-TS HLS request identities/layouts are distinct.
- FFmpeg command planning emits the expected fMP4 muxer flags when requested.
- HLS segment serving reports the right content type for `.ts`, `.m4s`, and init
  segments.
- Adaptive variant policy is preserved as non-executable or split to a follow-on
  without silent overclaiming.
- Focused transcode/server gates pass and evidence is recorded.
