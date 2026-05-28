# Transcode Output Shape, HLS Manifest, And Ladder Runtime - Evidence And Gates

Status: Active
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

## Notes

- Keep Public/Admin surfaces redaction-safe.
- Preserve MPEG-TS and fMP4 single-variant behavior while adding adaptive.
- Do not copy Jellyfin or FFmpeg reference implementation material.
