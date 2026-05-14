# Phase 4.2: Playback Decision and Direct Play API

## Goal

Add the first playback surface without committing to a full transcoding
pipeline. Taru can now decide whether a source should direct play, remux, or
transcode, and it can serve local direct-play media with HTTP byte ranges.

## Implemented Shape

### Playback Decision

`taru-streaming` now owns the first playback decision model:

- `ClientPlaybackCapabilities`
- `PlaybackDecision`
- `DirectPlayPlan`
- `PlaybackMode::{DirectPlay, Remux, Transcode}`

The decision path uses source file extension, optional probe data, and client
capabilities. Compatible MP4/WebM-style sources direct play. Unsupported
containers with compatible codecs request remux. Unsupported codecs request
transcode. No FFmpeg process is started in this phase.

### Direct Play Route

The server exposes:

```text
GET /sources/{source_id}/playback/decision
GET /sources/{source_id}/stream
HEAD /sources/{source_id}/stream
```

The stream route serves local sources and supports:

- full response with `200 OK`;
- byte-range response with `206 Partial Content`;
- range preflight through `HEAD`;
- `Accept-Ranges: bytes`;
- `Content-Range`;
- `Content-Length`;
- conservative MIME inference for common video containers.

Phase 4.2.1 moves direct play response planning into `taru-streaming` and
`taru-server::app`. The HTTP handler now maps headers into a range request,
calls the service plan, and streams the requested body. Malformed,
unsupported multi-range, and unsatisfiable ranges return
`416 Range Not Satisfiable` with `Content-Range: bytes */{total_len}`.

The implementation currently requires a local path hint from `taru-vfs`.
Remote source staging and byte-range cache remain future work.

## Non-Goals

- No HLS output yet.
- No FFmpeg remux/transcode session manager yet.
- No hardware acceleration detection yet.
- No subtitle selection or stream selection yet.
- No remote-source direct streaming yet beyond the VFS boundary preparation.

## Validation

Coverage added or updated for:

- direct-play decision for compatible MP4 sources;
- remux decision for unsupported containers with compatible codecs;
- HTTP range parsing and resolution;
- direct play response planning for full, empty, partial, and invalid ranges;
- source lookup by ID;
- source-level playback decision route;
- direct stream route returning exact partial bytes and `Content-Range`;
- direct stream `HEAD`, zero-byte, unsatisfiable range, and multi-range
  rejection behavior.

Required gates:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```
