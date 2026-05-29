# Playback Planner Transcode Value Vocabulary

Status: Completed
Last updated: 2026-05-29

This fearless refactor lane removes transcode execution value types from the
playback planner public surface. `nako-playback` should own planner-facing
values for remux/transcode output shape, HLS output preference, stream
selection, output constraints, and subtitle strategy. `nako-server` should map
those values into `nako-transcode` execution requests.

## Why Now

`playback-api-transcode-boundary-cleanup` removed the direct
`nako-api -> nako-transcode` edge, but PATB-030 found that `nako-playback`
still exposes transcode-owned values through planner records. That keeps the
planner tied to execution vocabulary before HDR, audio compatibility,
subtitle burn-in, and scheduler work add more pressure.

## Target Result

- `nako-playback` no longer declares a direct dependency on `nako-transcode`.
- Playback planner records use playback-owned value types.
- `nako-server` maps playback values into `nako-transcode` execution values.
- Existing playback decisions, request identity strings, Public Client JSON,
  and Admin diagnostics remain behavior-compatible.

## Architecture References

- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
- `docs/adr/0044-playback-capability-profile-planner.md`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
- `docs/workstreams/playback-api-transcode-boundary-cleanup/`
