# HLS Alternate Audio Renditions

Status: Closed
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

## Outcome

Closed after making selected HLS audio stream mapping executable. HLS command
planning now maps requested source audio stream `N` as `0:N` for single-variant
and adaptive HLS, while preserving the optional `0:a:0?` fallback and no-audio
adaptive behavior.

Nako still does not emit `EXT-X-MEDIA:TYPE=AUDIO`: that would be dishonest
until a follow-on lane adds multi-audio source facts, `HlsAudioRendition`
identity, audio-only HLS artifacts, artifact serving, and master playlist audio
group authoring.
