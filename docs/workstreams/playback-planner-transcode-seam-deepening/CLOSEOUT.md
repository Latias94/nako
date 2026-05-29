# Playback Planner Transcode Seam Deepening - Closeout

Closed: 2026-05-29

## Summary

This lane moved remux/HLS playback `TranscodeProfile` construction out of
`nako-playback` and into `nako-transcode` without changing runtime behavior or
wire contracts.

## Shipped

- Added transcode-owned `PlaybackRemuxProfileRequest` and
  `PlaybackHlsProfileRequest`.
- Added `build_playback_remux_profile` and `build_playback_hls_profile`.
- Deleted `PlaybackTargetProfile` methods that directly built
  `TranscodeProfile` values.
- Updated server remux/HLS composition and request identity test helpers to use
  the transcode-owned builders.
- Kept playback planner tests focused on planning facts rather than execution
  profile assembly.

## Verification

- `python3 -m json.tool docs/workstreams/playback-planner-transcode-seam-deepening/WORKSTREAM.json`
- `cargo nextest run -p nako-transcode profile --no-fail-fast`
- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## Follow-Ons

- HLS seek/restart lifecycle.
- HDR tone mapping pipeline.
- Audio downmix and normalization policy.
- Subtitle burn-in policy.
- Runtime resource scheduler and host capacity diagnostics.

