# HLS Alternate Audio Renditions

Status: Active
Last updated: 2026-05-29

Durable fearless refactor lane for making HLS audio track handling honest
before Nako advertises alternate audio renditions through master playlists.

The prior HLS media-rendition lanes made selected subtitle WebVTT sidecars
servable and discoverable. Alternate audio has a sharper correctness bar:
clients must never be offered an audio rendition that Nako did not actually
generate, and selected-audio playback must map the requested source stream
before a richer audio group model is exposed.

The first executable slice is selected audio integrity for HLS:

- make HLS FFmpeg stream mapping consume `TranscodeTrackSelection.audio_stream`;
- preserve optional-audio behavior for sources without audio;
- keep request identity, session reuse, adaptive fMP4 no-audio behavior, and
  Public/Admin redaction gates green;
- leave true alternate audio sidecar generation as a follow-on task unless the
  selected-audio slice proves the required artifact contract is already ready.

Out of scope for the first slice: audio-only HLS sidecar encoding for every
source audio track, codec-copy alternate audio passthrough, mixed codec groups,
language-preference UI, and live/LL-HLS behavior.
