# HLS Audio Sidecar Artifacts

Status: Active
Last updated: 2026-05-29

Durable fearless refactor lane for generating and publishing real HLS audio
sidecar renditions. This follows the selected-audio stream-map fix and the
subtitle media-group authoring lane.

The first executable slice is deliberately narrow:

- only sources with multiple audio streams get HLS audio sidecar renditions;
- generated audio sidecars are AAC/ADTS HLS playlists and segments;
- master playlists emit `EXT-X-MEDIA:TYPE=AUDIO` only for generated artifacts;
- selected-audio main mux behavior remains intact for compatibility and reuse.

Out of scope for this slice: codec-copy audio sidecars, mixed audio codec
groups, language preference UI, video-only variant generation, DRM, LL-HLS, and
replacing FFmpeg CLI execution.
