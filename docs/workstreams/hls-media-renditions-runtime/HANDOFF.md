# HLS Media Renditions Runtime Handoff

Status: Active
Last updated: 2026-05-28

## Current State

This workstream is open. Nako's HLS runtime now has source-aware adaptive video
ladders and audio-presence-aware adaptive maps, but it still lacks a typed
media rendition model for selected subtitles or alternate audio.

## Next Task

Start with HMR-020.

Recommended order:

1. Inspect `TranscodeTrackSelection`, `TranscodeRequirementStreams`,
   `TranscodePipelineSourceFacts`, `HlsArtifactManifest`, and FFmpeg HLS command
   planning.
2. Decide whether selected WebVTT subtitles can be implemented as the first
   executable slice without adding broad subtitle extraction policy.
3. Add the smallest typed media rendition plan and request identity hook needed
   for session reuse and artifact reconstruction.

## Validation To Preserve

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-transcode ffmpeg --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

## Cautions

- Keep subtitle and alternate-audio decisions deterministic across request
  identity, staging, and artifact reconstruction.
- Do not broaden this lane into LL-HLS, DRM, subtitle OCR, or full alternate
  audio UX.
- Preserve source-aware adaptive fMP4 and no-audio stream-map behavior.
