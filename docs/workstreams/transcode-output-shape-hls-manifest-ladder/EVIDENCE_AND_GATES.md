# Transcode Output Shape, HLS Manifest, And Ladder Runtime - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Gate Set

### Slice Gates

```bash
cargo nextest run -p nako-transcode profile --no-fail-fast
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-transcode ffmpeg --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### Closeout Gates

```bash
python3 -m json.tool docs/workstreams/transcode-output-shape-hls-manifest-ladder/WORKSTREAM.json
cargo fmt --all -- --check
git diff --check
```

## Evidence Log

- 2026-05-28 TOSHL-010: Opened lane after
  `executable-hls-fmp4-runtime-boundary` closed with adaptive HLS split as the
  next runtime follow-on.
- 2026-05-28 TOSHL-020: Replaced transitional `TranscodeProfile` output fields
  with typed `TranscodeOutputShape`, updated server HLS extraction, and
  preserved request identity semantics.
  - `cargo nextest run -p nako-transcode profile --no-fail-fast` passed: 9
    tests.
  - `cargo nextest run -p nako-playback --no-fail-fast` passed: 19 tests.
  - `cargo nextest run -p nako-server hls --no-fail-fast` passed: 24 tests.
- 2026-05-28 TOSHL-030: Introduced `HlsArtifactManifest` and
  `TranscodeArtifactSet`, moved FFmpeg HLS requests to manifest-shaped
  artifacts, and made server artifact serving use manifest rules for playlist,
  init segment, media segments, content type, cleanup candidates, and reuse.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed.
  - `cargo nextest run -p nako-transcode hls --no-fail-fast` passed: 21 tests.
  - `cargo nextest run -p nako-server hls --no-fail-fast` passed: 24 tests.
  - `cargo nextest run -p nako-server playback --no-fail-fast` passed: 92
    tests.
  - `cargo nextest run -p nako-playback --no-fail-fast` passed: 19 tests.
- 2026-05-28 TOSHL-040: Implemented the first executable adaptive HLS fMP4
  ladder slice with typed renditions, `hls_adaptive` request identity, FFmpeg
  master/variant command planning, server adaptive staging, manifest-derived
  artifact serving, and master playlist rewrite coverage.
  - `cargo nextest run -p nako-transcode ffmpeg --no-fail-fast` passed: 29
    tests.
  - `cargo nextest run -p nako-transcode transcode_profile --no-fail-fast`
    passed: 9 tests.
  - `cargo nextest run -p nako-server hls --no-fail-fast` passed: 28 tests.
  - `cargo nextest run -p nako-server playback --no-fail-fast` passed: 96
    tests.
  - `cargo nextest run -p nako-playback --no-fail-fast` passed: 19 tests.
- 2026-05-28 TOSHL-050: Closed the workstream after verifying the requested
  target state and recording residual adaptive breadth as follow-ons.
  - `python3 -m json.tool docs/workstreams/transcode-output-shape-hls-manifest-ladder/WORKSTREAM.json`
    passed.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed.
  - `c3bc1522 feat(hls): execute adaptive ladder runtime slice` committed the
    final adaptive runtime slice.

## Notes

- Public/Admin redaction remained covered by the `nako-server playback` gate,
  including playback session, runtime, and support-evidence redaction tests.
- MPEG-TS and fMP4 single-variant behavior remained covered while adaptive
  fMP4 was added as a distinct executable shape.
- No Jellyfin or FFmpeg reference implementation material was copied.
