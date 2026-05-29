# HLS Alternate Audio Renditions - Design

Status: Active
Last updated: 2026-05-29

## Problem

Nako can carry `requested_audio_stream` through playback profile identity and
transcode request identity, but the executable HLS FFmpeg command currently maps
the first audio stream with `0:a:0?`. That means a request for another source
audio stream can reuse a distinct session identity while still producing the
wrong audio bytes.

Alternate audio rendition support would amplify that bug. A master playlist can
advertise `EXT-X-MEDIA:TYPE=AUDIO`, but Nako must first prove that HLS command
planning, artifact identity, and session reconstruction agree on which source
audio stream is being generated.

## Intent

Deepen the HLS audio boundary in the same direction as subtitles:

```text
PlaybackPreferenceContext
  -> TranscodeTrackSelection
  -> HLS command stream mapping
  -> HLS artifact manifest and request identity
  -> master playlist authoring
```

Do not publish alternate audio groups until Nako can generate and serve the
referenced audio playlists. The first stage makes selected audio exact; later
stages can add `HlsAudioRendition` and audio sidecar artifacts without lying to
clients.

## Refactor Brief

- **Intent:** remove the hidden first-audio-stream assumption before adding
  alternate audio surface area.
- **Scope:** `nako-transcode` HLS stream-map command planning, selected audio
  tests, `nako-server` playback tests, and workstream evidence.
- **Deletion plan:** delete fixed `0:a:0?` HLS mapping where a selected source
  audio stream is known.
- **Boundary plan:** keep audio selection in `TranscodeTrackSelection`; keep
  HLS command construction in `nako-transcode`; keep playlist authoring in
  `nako-server`.
- **Testing plan:** unit-test HLS command argv for selected audio stream maps;
  integration-test HLS source selected-audio requests so request identity and
  runner argv agree.
- **Risk plan:** preserve optional audio mapping when no selected stream exists;
  do not regress adaptive no-audio stream maps; do not expose `TYPE=AUDIO`
  master tags until audio sidecar artifacts exist.
- **Workflow plan:** one durable workstream. Close the selected-audio mapping
  slice if true alternate audio sidecars need a separate larger lane.

## Target State

Selected audio integrity:

- default HLS still maps `0:a:0?`;
- requested source audio stream `N` maps `0:N`;
- adaptive HLS maps the selected audio stream once per video rendition;
- no-audio adaptive plans still omit audio maps and audio encoders.

Future alternate audio:

- `HlsMediaRenditionPlan` grows audio rendition facts only when FFmpeg command
  planning can generate matching audio-only HLS playlists;
- HLS master authoring emits `TYPE=AUDIO` only for artifacts in the manifest;
- ticket decoration and playback-session URI rewrite reuse the quoted-URI
  support already added for subtitle media groups.

## Non-Goals

- Do not emit `EXT-X-MEDIA:TYPE=AUDIO` for audio that is still muxed only into
  video variant playlists.
- Do not implement language-preference selection or default-audio heuristics in
  this first slice.
- Do not replace FFmpeg CLI execution with rsmpeg/libav bindings.
- Do not copy Jellyfin, FFmpeg, or rsmpeg source, schemas, tests, comments, or
  assets.

## Closeout Condition

This lane can close when selected HLS audio mapping is executable and verified,
or when evidence shows audio sidecar generation is ready enough to continue to
`TYPE=AUDIO` media groups in the same lane.
