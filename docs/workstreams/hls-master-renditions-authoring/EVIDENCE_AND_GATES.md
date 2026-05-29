# HLS Master Renditions Authoring - Evidence And Gates

Status: Closed
Last updated: 2026-05-29

## Gate Set

### Slice Gates

```bash
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### Closeout Gates

```bash
python3 -m json.tool docs/workstreams/hls-master-renditions-authoring/WORKSTREAM.json
cargo fmt --all -- --check
git diff --check
```

## Evidence Log

- 2026-05-29 HMA-010: Opened the durable fearless refactor lane after
  `hls-media-renditions-runtime` closed with selected subtitle WebVTT sidecar
  artifact planning, serving, and reuse.
- 2026-05-29 HMA-020/HMA-030: Added server-owned HLS entry playlist authoring
  from `HlsArtifactManifest`, generated single-variant master entry playlists
  for selected subtitle sidecars, enriched adaptive fMP4 masters with subtitle
  media groups, and rewrote `EXT-X-MEDIA:URI` attributes.
- 2026-05-29 HMA-030: Extended browser and renderer HLS ticket decoration to
  quoted media group URIs so subtitle playlists stay authorized.
- 2026-05-29 HMA-040: `cargo nextest run -p nako-transcode hls
  --no-fail-fast` passed, 27 tests.
- 2026-05-29 HMA-040: `cargo nextest run -p nako-server hls --no-fail-fast`
  passed, 40 tests.
- 2026-05-29 HMA-040: `cargo nextest run -p nako-server playback
  --no-fail-fast` passed, 108 tests.
- 2026-05-29 HMA-040: `python3 -m json.tool
  docs/workstreams/hls-master-renditions-authoring/WORKSTREAM.json`,
  `cargo fmt --all -- --check`, and `git diff --check` passed.

## Notes

- Public/Admin redaction must remain covered by `nako-server playback` before
  closeout.
- Do not copy Jellyfin, FFmpeg, or rsmpeg reference implementation material.
