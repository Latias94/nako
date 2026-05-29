# HLS Audio Sidecar Artifacts - Design

Status: Closed
Last updated: 2026-05-29

## Problem

Nako now honors selected HLS audio stream mapping, but it still cannot publish
alternate audio tracks because no audio-only HLS artifacts exist. Advertising
`EXT-X-MEDIA:TYPE=AUDIO` before generating those artifacts would create broken
client URLs.

## Intent

Make audio rendition publication artifact-driven:

```text
MediaProbeResult audio streams
  -> HlsAudioRendition facts
  -> FFmpeg audio-only HLS sidecar outputs
  -> HlsArtifactManifest serving
  -> TYPE=AUDIO master playlist authoring
```

The server may keep muxing selected audio into the primary video HLS output in
this slice. The invariant is stricter: every `TYPE=AUDIO` URI in the public
playlist must resolve to a generated and servable artifact.

## Refactor Brief

- **Intent:** remove the last blocker to truthful HLS alternate audio
  publication by making audio sidecars first-class artifacts.
- **Scope:** `nako-transcode` media rendition identity/artifact/FFmpeg planning,
  `nako-server` HLS staging, artifact reconstruction, playlist authoring, and
  focused playback tests.
- **Deletion plan:** avoid ad hoc audio group playlist injection; delete any
  assumption that media renditions are subtitle-only.
- **Boundary plan:** `nako-transcode` owns audio rendition naming, identity,
  artifact membership, and FFmpeg output args; `nako-server` owns choosing
  which source audio streams become sidecars and publishing them in public
  playlists.
- **Testing plan:** unit-test identity round trip, artifact membership, FFmpeg
  audio sidecar args, master playlist authoring, and server HLS source output.
- **Risk plan:** emit audio sidecars only for multi-audio sources; do not
  advertise audio groups without matching artifacts; preserve no-audio adaptive
  behavior and selected-audio main mux.
- **Workflow plan:** one durable workstream with a single executable slice,
  then close or split video-only variant cleanup as a follow-on.

## Target Flow

- Probe sees multiple audio streams.
- Nako creates dense `HlsAudioRendition` entries.
- Selected audio stream is marked default.
- FFmpeg emits `audio_N.m3u8` and `audio_N_00000.aac` sidecar artifacts.
- Public master playlist emits `TYPE=AUDIO` tags and attaches `AUDIO=`.
- Segment route serves audio playlist and `.aac` segments.

## Non-Goals

- Do not remove selected audio from the main video HLS output in this slice.
- Do not implement codec-copy or source-codec-preserving audio sidecars.
- Do not choose default audio from user language preferences yet.
- Do not replace FFmpeg CLI with rsmpeg/libav bindings.
- Do not copy Jellyfin, FFmpeg, or rsmpeg source, schemas, tests, comments, or
  assets.

## Closeout Condition

This lane can close when generated audio sidecar artifacts are represented in
`HlsArtifactManifest`, produced by FFmpeg command planning, served by existing
HLS segment routes, advertised in public master playlists, and verified by
focused Rust gates.

## Closeout Summary

The lane is closed. Multi-audio HLS sources now get dense typed
`HlsAudioRendition` entries, generated AAC/ADTS audio sidecar playlists and
segments, manifest-backed serving, and public `TYPE=AUDIO` master playlist
publication. Selected audio remains muxed into the main HLS output.
