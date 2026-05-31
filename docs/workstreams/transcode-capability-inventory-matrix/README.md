# Transcode Capability Inventory Matrix

Status: Active
Last updated: 2026-05-31

This workstream deepens `nako-transcode` capability inventory without changing
HLS runtime selection or FFmpeg command planning. It captures more observable
FFmpeg/host facts for future HDR tone mapping, HEVC/AV1, subtitle burn-in, and
operator diagnostics while keeping HDR `HTP-030` free to own the first
software-first media-output path.

Planner-approved lane: `playback-transcode`.

First shipped baseline: `TCIM-020` records optional bitstream-filter evidence
without changing pipeline selection.

Second shipped baseline: `TCIM-030` records optional decoder, encoder, filter,
tone-map, and subtitle burn-in evidence without changing pipeline selection.

Next executable task: `TCIM-040`, closeout and follow-on split.

Read before implementation:

- `CONTEXT.md`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
- `docs/adr/0046-ffmpeg-probe-inventory.md`
- `docs/adr/0048-playback-transcode-startup-degradation.md`
- `docs/architecture/PLAYBACK.md`
- `docs/workstreams/transcode-capability-inventory-matrix/CONTEXT.jsonl`

Do not expand this workstream into pipeline selection, HDR FFmpeg filter
execution, HLS lifecycle, resource admission, server orchestration, or release
packaging gates.
