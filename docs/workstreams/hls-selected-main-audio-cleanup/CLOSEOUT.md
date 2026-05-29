# HLS Selected Main Audio Cleanup Closeout

Date: 2026-05-29
Status: Completed

## Result

The lane removed selected-main-audio duplication for sidecar-capable HLS
outputs.

- Generated audio sidecar outputs now make the primary HLS video output
  video-only.
- Adaptive HLS sidecar-capable variants use video-only `-var_stream_map`
  entries while public playlist authoring attaches the generated `TYPE=AUDIO`
  group.
- Single-audio and no-sidecar outputs keep muxed audio behavior.
- HLS request variant identity records `hls-main-output:v1;main_audio=false`
  for generated audio sidecar outputs, preventing reuse of older duplicated
  output shapes.
- Existing browser/renderer HLS routes, segment serving, and session reuse
  contracts remain stable.

## Deferred Follow-ons

- Language preference and default audio selection policy.
- Codec-copy or codec-aware audio sidecar generation.
- LL-HLS/CMAF, DASH/CMAF, DRM, and key delivery.
- Player-specific fallback negotiation for clients that cannot consume HLS
  audio groups.

## Verification

- `cargo nextest run -p nako-transcode hls --no-fail-fast` (42 passed, 38
  skipped)
- `cargo nextest run -p nako-server hls --no-fail-fast` (53 passed, 422
  skipped)
- `cargo nextest run -p nako-server playback --no-fail-fast` (132 passed, 343
  skipped)
- `cargo fmt --all -- --check`
- `python3 -m json.tool docs/workstreams/hls-selected-main-audio-cleanup/WORKSTREAM.json`
- `git diff --check`
