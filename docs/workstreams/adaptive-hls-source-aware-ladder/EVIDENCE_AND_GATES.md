# Adaptive HLS Source-Aware Ladder Runtime - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Gate Set

### Slice Gates

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-transcode transcode_profile --no-fail-fast
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-transcode ffmpeg --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### Closeout Gates

```bash
python3 -m json.tool docs/workstreams/adaptive-hls-source-aware-ladder/WORKSTREAM.json
cargo fmt --all -- --check
git diff --check
```

## Evidence Log

- 2026-05-28 AHSL-010: Opened the durable fearless refactor lane after
  `transcode-output-shape-hls-manifest-ladder` closed with fixed ladder and
  no-audio stream-map support split as follow-ons.

## Notes

- Public/Admin redaction must remain covered by focused `nako-server playback`
  gates before closeout.
- MPEG-TS and fMP4 single-variant behavior must remain unaffected while
  adaptive fMP4 becomes source-aware.
- No Jellyfin, FFmpeg, or rsmpeg reference implementation material may be
  copied.
