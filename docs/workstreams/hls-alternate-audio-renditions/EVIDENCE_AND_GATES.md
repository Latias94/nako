# HLS Alternate Audio Renditions - Evidence And Gates

Status: Closed
Last updated: 2026-05-29

## Gate Set

### Slice Gates

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
```

### Closeout Gates

```bash
python3 -m json.tool docs/workstreams/hls-alternate-audio-renditions/WORKSTREAM.json
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Evidence Log

- 2026-05-29 HAA-010: Opened the durable fearless refactor lane after closing
  HLS subtitle master playlist authoring. Found that selected audio request
  identity exists, but executable HLS FFmpeg mapping still defaults to
  `0:a:0?`.
- 2026-05-29 HAA-020: Added `HlsRequest.track_selection` and made HLS FFmpeg
  stream mapping consume `TranscodeTrackSelection.audio_stream`.
- 2026-05-29 HAA-020: `cargo nextest run -p nako-transcode hls
  --no-fail-fast` passed, 29 tests.
- 2026-05-29 HAA-020: `cargo nextest run -p nako-server hls --no-fail-fast`
  passed, 41 tests.
- 2026-05-29 HAA-030: Closed without emitting `TYPE=AUDIO`; true alternate
  audio needs a follow-on audio sidecar artifact lane before master playlist
  audio groups can be truthful.
- 2026-05-29 HAA-030: `cargo nextest run -p nako-server playback
  --no-fail-fast` passed, 109 tests.
- 2026-05-29 HAA-030: `python3 -m json.tool
  docs/workstreams/hls-alternate-audio-renditions/WORKSTREAM.json`,
  `cargo fmt --all -- --check`, and `git diff --check` passed.

## Notes

- Do not emit `EXT-X-MEDIA:TYPE=AUDIO` unless matching audio HLS artifacts are
  generated and servable.
- Do not copy Jellyfin, FFmpeg, or rsmpeg reference implementation material.
