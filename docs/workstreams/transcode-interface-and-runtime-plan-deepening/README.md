# Transcode Interface And Runtime Plan Deepening

Status: Closed
Last updated: 2026-05-31

This workstream deepened the `nako-transcode` Interface before HDR tone mapping
adds more playback/transcode pressure. `nako-server` no longer assembles the
low-level HLS runtime plan or raw FFmpeg request details for playback HLS/remux
execution; those paths now enter through transcode-owned runtime and execution
planner Interfaces.

Planner-approved lane: `playback-transcode`.

Closed result: `TIRP-020`, `TIRP-030`, and `TIRP-040` are complete.

Read before implementation:

- `CONTEXT.md`
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/transcode-interface-and-runtime-plan-deepening/CONTEXT.jsonl`

Do not reopen this workstream for HDR tone mapping, HLS lifecycle ownership,
resource admission unification, or broad Jellyfin-class hardware matrices.
Those are follow-on lanes. HDR `HTP-030` may now start from current `main`
using the transcode-owned runtime and execution planner Interfaces.
