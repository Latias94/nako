# HLS Media Renditions Runtime - Evidence And Gates

Status: Closed
Last updated: 2026-05-29

## Gate Set

### Slice Gates

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-transcode ffmpeg --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### Closeout Gates

```bash
python3 -m json.tool docs/workstreams/hls-media-renditions-runtime/WORKSTREAM.json
cargo fmt --all -- --check
git diff --check
```

## Evidence Log

- 2026-05-28 HMR-010: Opened the durable fearless refactor lane after
  `adaptive-hls-source-aware-ladder` closed with source-aware adaptive video
  ladders and no-audio stream maps.
- 2026-05-29 HMR-020: Added typed HLS media rendition and request-variant
  identity boundaries for selected subtitle sidecar artifacts.
  - `cargo nextest run -p nako-transcode hls --no-fail-fast` passed: 27 tests.
  - `cargo nextest run -p nako-playback --no-fail-fast` passed: 19 tests.
- 2026-05-29 HMR-030: Implemented the first executable selected-subtitle HLS
  WebVTT slice in FFmpeg command planning and HLS artifact allow-listing.
  - `cargo nextest run -p nako-transcode ffmpeg --no-fail-fast` passed: 31
    tests.
  - `cargo nextest run -p nako-server hls --no-fail-fast` passed: 35 tests.
- 2026-05-29 HMR-040: Integrated selected subtitle media renditions into server
  HLS staging, persisted request identity, artifact reconstruction, playlist
  rewriting coverage, session reuse, and playback route coverage.
  - `cargo nextest run -p nako-server playback --no-fail-fast` passed: 103
    tests.
  - `python3 -m json.tool docs/workstreams/hls-media-renditions-runtime/WORKSTREAM.json`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed.

## Notes

- Public/Admin redaction remains covered by `nako-server playback` before
  closeout.
- Do not copy Jellyfin, FFmpeg, or rsmpeg reference implementation material.
