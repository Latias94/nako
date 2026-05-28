# Adaptive HLS Source-Aware Ladder Runtime

Status: Active
Last updated: 2026-05-28

Durable fearless refactor lane for making Nako's adaptive HLS fMP4 runtime
source-aware after the first executable ladder slice.

The previous `transcode-output-shape-hls-manifest-ladder` lane proved adaptive
master/variant playlist execution with a fixed two-rendition ladder. This lane
removes the next two runtime shortcuts before adaptive playback becomes a
realistic self-hosted media-server path:

- derive adaptive renditions from source video facts and client output
  constraints instead of using a fixed ladder that can upscale or over-encode;
- make adaptive FFmpeg command planning support sources without audio instead
  of assuming every variant has a mapped audio stream.

The lane keeps adaptive MPEG-TS, alternate audio, subtitle renditions, LL-HLS,
CMAF encryption, DRM, and a second engine adapter outside this scope unless
they are needed to preserve the source-aware fMP4 contract.
