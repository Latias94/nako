# ADR 0052: HLS Runtime And Media Engine Boundary

Status: accepted
Date: 2026-05-29

## Context

Nako is deepening playback/transcode toward a self-hosted media server that can
eventually compete with Jellyfin/Plex-class playback behavior. Recent HLS work
made fMP4, adaptive ladders, selected audio mapping, subtitle sidecars, and
audio sidecars executable.

The next architectural risk is ambiguity about where the media engine ends and
where Nako's runtime begins. One model starts FFmpeg lazily when a segment is
requested. Another starts a transcode session when playback is planned and lets
segment routes wait for generated artifacts. A third would bind directly to
libavformat/libavcodec through Rust FFI and make Nako own muxing internals.

Nako needs a clear boundary before adding seek restart, tone mapping, DASH,
remote workers, or richer GPU scheduling.

## Decision

Nako will keep a **FFmpeg CLI-first media engine boundary** for playback
transcode and HLS.

Rust owns:

- playback decision and capability planning;
- transcode request identity;
- FFmpeg command planning;
- process lifecycle, cancellation, timeout, and runtime metrics;
- resource policy and hardware fallback;
- HLS artifact manifests and safe serving;
- playlist rewriting and browser/renderer transport tickets;
- session reuse, cleanup, and public/admin error redaction.

FFmpeg/ffprobe own:

- probing container and stream facts;
- decode, filter, encode, mux, and segment work;
- HLS media playlist and segment generation;
- hardware encoder/decoder integration exposed through CLI capabilities.

Nako will publish HLS playlists, media groups, and segment URLs only when the
typed `HlsArtifactManifest` can identify the artifact as part of the session.
Generated media renditions such as subtitle and audio sidecars must be part of
request-variant identity so reused sessions can reconstruct the same artifact
allow-list.

The current HLS runtime model is session-started transcode:

```text
HLS source request
  -> playback decision and request identity
  -> HLS staging layout and artifact manifest
  -> FFmpeg process
  -> playlist/segment routes serve manifest-approved files
```

Segment routes may wait briefly for running sessions, but they do not currently
own lazy process creation. Seeking, segment-window restart, and lazy segment
generation remain follow-on design work.

## Consequences

- Nako can move quickly while relying on FFmpeg's mature muxers, codecs, and
  hardware integrations.
- Command planning stays testable as structured Rust data instead of shell
  string assembly.
- Public HLS URLs remain tied to typed manifests rather than directory listing
  or ad hoc filename checks.
- Reused transcode sessions can rebuild artifact membership from persisted
  request identity.
- Future rsmpeg/libav experimentation must prove a concrete benefit before it
  can enter the core playback path.
- Seek and lazy segment generation need their own workstream because they alter
  process lifecycle, session reuse, and cleanup semantics.

## Alternatives Considered

- **Lazy FFmpeg start from segment miss:** useful for seek-heavy playback, but
  it makes segment routes responsible for process orchestration and complicates
  session identity. Deferred until the current session-started runtime is fully
  mature.
- **Direct libav/rsmpeg media engine:** powerful but increases unsafe memory
  and codec-lifecycle risk. Deferred until there is a measured reason that
  FFmpeg CLI cannot satisfy.
- **Static HLS directory serving:** simple, but it cannot enforce Nako's
  artifact allow-list, ticket rewriting, redaction, and session reuse rules.
- **Publish media groups from playlist strings alone:** rejected because HLS
  `TYPE=AUDIO` or `TYPE=SUBTITLES` must not advertise URLs that cannot be
  served by the manifest.

## Related Workstreams

- `docs/workstreams/source-aware-transcode-runtime/`
- `docs/workstreams/executable-hls-fmp4-runtime-boundary/`
- `docs/workstreams/transcode-output-shape-hls-manifest-ladder/`
- `docs/workstreams/adaptive-hls-source-aware-ladder/`
- `docs/workstreams/hls-media-renditions-runtime/`
- `docs/workstreams/hls-master-renditions-authoring/`
- `docs/workstreams/hls-alternate-audio-renditions/`
- `docs/workstreams/hls-audio-sidecar-artifacts/`
- `docs/workstreams/media-server-architecture-progress-map/`
