# HLS Selected Main Audio Cleanup - Handoff

Status: Completed
Last updated: 2026-05-29

## Current State

HSMA-010 through HSMA-040 are complete. The lane is closed. It follows the
closed `hls-audio-sidecar-artifacts` workstream, which generated audio-only HLS
sidecar artifacts and public `TYPE=AUDIO` groups while intentionally preserving
selected audio in the main video mux.

This lane removed that duplication only for sidecar-capable multi-audio HLS
outputs. Single-audio and no-sidecar outputs keep current behavior.

## Active Task

None. Open a new workstream for language preference policy, codec-aware audio
sidecars, LL-HLS/DASH/DRM, or player-specific fallback.

## Decisions Since Opening

- Keep this lane focused on selected-main-audio duplication removal.
- Do not add language preference policy, codec-copy sidecars, LL-HLS, DASH,
  DRM/key delivery, or player-specific fallback negotiation.
- Keep FFmpeg CLI as the media engine boundary.
- Preserve `TYPE=AUDIO` publication only for manifest-backed generated audio
  sidecar artifacts.
- HSMA-020 proved the current duplication in `nako-transcode`:
  `ffmpeg_builder_duplicates_selected_audio_for_hls_sidecars` maps selected
  audio into the primary HLS output and again as an audio sidecar, while
  `ffmpeg_builder_duplicates_selected_audio_for_adaptive_hls_sidecars` maps it
  once per adaptive rendition plus once as a sidecar.
- HSMA-030 changed the output shape: generated audio sidecar outputs now make
  the primary HLS video output video-only, and request variant identity records
  `hls-main-output:v1;main_audio=false`.
- HSMA-040 closed the lane after fresh transcode/server HLS/playback gates.

## Blockers

- None.

## Next Recommended Action

- Commit the verified lane.
- Open follow-ons only when product scope requires language preferences,
  codec-aware sidecars, LL-HLS/DASH/DRM, or player-specific fallback.
