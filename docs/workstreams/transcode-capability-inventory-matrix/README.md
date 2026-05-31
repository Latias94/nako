# Transcode Capability Inventory Matrix

Status: Closed
Last updated: 2026-05-31

This workstream deepened `nako-transcode` capability inventory without changing
HLS runtime selection or FFmpeg command planning. It captures more observable
FFmpeg/host facts for future HDR tone mapping, HEVC/AV1, subtitle burn-in, and
operator diagnostics while keeping executable policy and command planning in
follow-on lanes.

Planner-approved lane: `playback-transcode`.

Shipped tasks:

- `TCIM-020` recorded optional bitstream-filter evidence without changing
  pipeline selection.
- `TCIM-030` recorded optional decoder, encoder, filter, tone-map, and
  subtitle burn-in evidence without changing pipeline selection.

Read before follow-on implementation:

- `CONTEXT.md`
- `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
- `docs/adr/0046-ffmpeg-probe-inventory.md`
- `docs/adr/0048-playback-transcode-startup-degradation.md`
- `docs/architecture/PLAYBACK.md`
- `docs/workstreams/transcode-capability-inventory-matrix/CONTEXT.jsonl`

Open a new workstream before expanding into pipeline selection, HDR FFmpeg
filter execution, HLS lifecycle, resource admission, server orchestration,
Public/Admin DTOs, or release packaging gates.
