# HLS Media Renditions Runtime - Design

Status: Active
Last updated: 2026-05-28

## Problem

Nako's HLS runtime can now execute single-variant and adaptive fMP4 video
output, but the HLS artifact model still has no first-class media rendition
shape. That creates three near-term problems:

- selected subtitles are currently omitted for HLS rather than represented as
  WebVTT artifacts or playlist rendition references;
- alternate audio cannot be expressed by the manifest boundary even when source
  facts identify more than one audio stream;
- request identity and session artifact reconstruction have no durable place to
  carry decisions that affect non-video HLS artifacts.

Without this boundary, every subtitle or audio feature will need ad hoc
playlist rewriting and artifact allow-list logic.

## Intent

Add a typed HLS media rendition boundary before implementing broad media-track
features. The first executable slice should prove that non-video HLS artifacts
can be planned, identified, served, cleaned up safely, and covered by playback
session reuse/redaction tests.

## Refactor Brief

- **Intent:** remove the assumption that HLS runtime artifacts are only video
  playlists, init segments, and video segments.
- **Scope:** `nako-transcode` HLS artifact manifest, profile/request identity,
  and FFmpeg command planning; `nako-playback` selected stream facts if needed;
  `nako-server` staging, artifact serving, playlist rewriting, and playback
  runtime tests.
- **Deletion plan:** retire string/pattern-only assumptions where HLS artifact
  allow-listing cannot distinguish video segments from media rendition
  artifacts.
- **Boundary plan:** keep FFmpeg argument assembly in `nako-transcode`; make
  server staging consume typed HLS rendition plans; keep persisted session
  reconstruction driven by request identity or manifest-shaped deterministic
  defaults.
- **Testing plan:** focused tests for manifest artifact allow-listing,
  FFmpeg command plans, playlist rewrite, session reuse, and broader playback
  redaction gates.
- **Risk plan:** keep adaptive fMP4 source-aware ladder and single-variant
  MPEG-TS/fMP4 paths green; if selected subtitle extraction requires a broader
  text-subtitle pipeline, split it after landing a typed manifest foundation.
- **Workflow plan:** one durable workstream with a small first executable
  rendition slice, then close or split follow-ons.

## Target Flow

```text
PlaybackTargetProfile / selected streams
  -> HlsMediaRenditionPlan
  -> TranscodeRequestIdentity request_variant
  -> HlsArtifactManifest
  -> HlsRequest / FfmpegCommandBuilder::hls
  -> HlsArtifactService / playlist rewrite
```

## First Slice Preference

Preferred first slice:

- selected subtitle stream becomes a single HLS WebVTT rendition artifact set;
- HLS manifest can identify subtitle playlist and VTT segment files;
- FFmpeg command planning can map the selected subtitle stream to WebVTT when
  the selected subtitle is compatible with FFmpeg's text-subtitle path;
- server artifact serving and playlist rewrite know those artifacts are part of
  the session.

Fallback first slice:

- land typed `HlsMediaRenditionPlan` and artifact reconstruction without
  executing subtitle extraction, then split extraction into a smaller follow-on
  once source subtitle formats and FFmpeg behavior are pinned down.

## Non-Goals

- Do not implement LL-HLS, CMAF encryption, or DRM.
- Do not implement full alternate-audio UX or source selection UI.
- Do not implement subtitle OCR or image-subtitle burn-in.
- Do not replace the FFmpeg CLI adapter with rsmpeg.
- Do not copy Jellyfin, FFmpeg, or rsmpeg source, schemas, tests, comments, or
  assets.

## Closeout Condition

This lane can close when:

- HLS media rendition decisions have a typed manifest/request identity boundary;
- the first executable rendition slice is implemented or explicitly split after
  a verified foundation;
- server HLS artifact serving and playlist rewrite remain deterministic;
- adaptive fMP4 source-aware, no-audio, and single-variant paths remain covered;
- focused Rust gates pass and evidence is recorded.
