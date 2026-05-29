# HLS Media Renditions Runtime Handoff

Status: Closed
Last updated: 2026-05-29

## Current State

This workstream is closed. Nako's HLS runtime now has a typed media rendition
model for the first selected subtitle WebVTT sidecar slice, and the adaptive
video ladder request-variant identity has been generalized so media rendition
decisions can share the same persisted session identity boundary.

## Next Task

Open a follow-on only when the next media-rendition target is ready.

Recommended follow-ons:

1. Author richer HLS master playlist media tags so selected subtitles and
   future alternate audio are advertised explicitly to clients.
2. Add full alternate audio stream planning and serving.
3. Decide how image subtitles should flow: burn-in, OCR, or explicit
   unsupported policy.

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
- Do not fold LL-HLS, DRM, subtitle OCR, or full alternate audio UX into a
  cleanup patch; each needs its own scoped lane.
- Preserve source-aware adaptive fMP4 and no-audio stream-map behavior.
