# HLS Audio Sidecar Artifacts - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Gate Set

### Slice Gates

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
```

### Closeout Gates

```bash
python3 -m json.tool docs/workstreams/hls-audio-sidecar-artifacts/WORKSTREAM.json
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Evidence Log

- 2026-05-29 HAS-010: Opened the durable fearless refactor lane after selected
  HLS audio stream mapping became executable.

## Notes

- Do not emit `TYPE=AUDIO` unless matching audio sidecar playlists and segments
  are generated and servable.
- Do not copy Jellyfin, FFmpeg, or rsmpeg reference implementation material.
