# HLS Media Renditions Runtime - Evidence And Gates

Status: Active
Last updated: 2026-05-28

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

## Notes

- Public/Admin redaction must remain covered by `nako-server playback` before
  closeout.
- Do not copy Jellyfin, FFmpeg, or rsmpeg reference implementation material.
