# Executable HLS fMP4 Runtime Boundary - Evidence And Gates

Status: Completed
Last updated: 2026-05-28

## Gate Set

### Focused Iteration Gates

```bash
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-transcode ffmpeg --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### Closeout Gates

```bash
python3 -m json.tool docs/workstreams/executable-hls-fmp4-runtime-boundary/WORKSTREAM.json
cargo fmt --all -- --check
git diff --check
```

## Evidence Log

- 2026-05-28 EHFR-010: Opened the executable HLS fMP4 runtime boundary lane
  after `playback-media-maturity-first-slices` landed planner vocabulary.
- 2026-05-28 EHFR-010: `python3 -m json.tool
  docs/workstreams/executable-hls-fmp4-runtime-boundary/WORKSTREAM.json` passed.
- 2026-05-28 EHFR-020: `HlsOutputRequirement` now travels from playback target
  profile into `HlsTranscodeProfile`, `TranscodeProfile`, request identity, and
  HLS runtime layout selection. Adaptive policy is rejected for the executable
  single-variant runtime slice instead of silently collapsing to MPEG-TS.
- 2026-05-28 EHFR-030: `HlsRequest` carries output requirements, fMP4 command
  planning emits `-hls_segment_type fmp4` and `-hls_fmp4_init_filename init.mp4`,
  and staging emits `.m4s` segment patterns when requested.
- 2026-05-28 EHFR-040: HLS playlist rewriting handles `#EXT-X-MAP` init segment
  URIs, artifact serving returns `video/mp4` for `.m4s` and init `.mp4`, and
  stale cleanup covers `.ts` and `.m4s` without deleting the init segment.
- 2026-05-28 EHFR-050: Fresh closeout gates passed:
  `cargo nextest run -p nako-playback --no-fail-fast`;
  `cargo nextest run -p nako-transcode ffmpeg --no-fail-fast`;
  `cargo nextest run -p nako-transcode --no-fail-fast`;
  `cargo nextest run -p nako-server hls --no-fail-fast`;
  `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.

## Notes

- Do not claim adaptive bitrate ladder support in this lane.
- Keep MPEG-TS as default behavior.
- Do not expose raw source paths, command lines, or host paths in Public/Admin
  surfaces.
