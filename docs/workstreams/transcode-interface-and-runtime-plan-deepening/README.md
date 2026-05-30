# Transcode Interface And Runtime Plan Deepening

Status: Active
Last updated: 2026-05-31

This workstream deepens the `nako-transcode` Interface before HDR tone mapping
adds more playback/transcode pressure. The goal is to stop `nako-server` from
assembling low-level HLS and FFmpeg request details, then ratchet the public
transcode surface so FFmpeg command details remain internal adapters.

Planner-approved lane: `playback-transcode`.

First executable task: `TIRP-020`.

Read before implementation:

- `CONTEXT.md`
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/transcode-interface-and-runtime-plan-deepening/CONTEXT.jsonl`

Do not expand this workstream into HDR tone mapping, HLS lifecycle ownership,
resource admission unification, or broad Jellyfin-class hardware matrices.
