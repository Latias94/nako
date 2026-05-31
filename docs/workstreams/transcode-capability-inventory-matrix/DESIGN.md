# Transcode Capability Inventory Matrix

Status: Active
Last updated: 2026-05-31

## Why This Lane Exists

`nako-transcode` now has stage-aware hardware capability records and execution
planner Interfaces, but the inventory remains focused on the first HLS
H.264/AAC path. Future tone mapping, HEVC/AV1 output, subtitle burn-in, and
operator diagnostics need more observable FFmpeg facts before policy selection
should try to consume them.

The risk is doing this backwards: adding pipeline branches or command filters
before the capability report can explain what the host can actually do.

## Target State

When this workstream closes:

- capability reporting can represent decoder, encoder, filter, tone-map,
  subtitle, and bitstream-filter evidence beyond the current H.264 baseline;
- the new facts are test-covered in `nako-transcode`;
- existing HLS pipeline selection and FFmpeg command planning remain unchanged;
- future HDR/hardware work has a richer report to consume without copying
  Jellyfin's model.

## In Scope

- `nako-transcode` FFmpeg probe inventory values;
- stage capability evidence values;
- tests for inventory/report construction;
- redaction-safe operator-facing strings when already present in transcode
  reports.

## Out Of Scope

- changing `TranscodePipelinePlanner` selection behavior;
- adding HDR-to-SDR FFmpeg filters;
- HEVC/AV1 output profile support;
- subtitle burn-in execution;
- server Admin routes or Public Client DTOs;
- release packaging or Docker GPU documentation.

## Architecture Direction

Keep this as an evidence-layer deepening. The Module should answer "what can
this host's FFmpeg build expose?" without deciding "which pipeline should this
request run?".

`TCIM-020` intentionally ships a narrow bitstream-filter baseline first because
it proves optional stage evidence can exist without influencing HLS pipeline
selection. Broader decoder, encoder, filter, tone-map, and subtitle inventory
coverage builds on that seam in `TCIM-030`, while keeping the new facts
optional and evidence-only.

The deletion test is: if the richer inventory were deleted, future pipeline
and HDR work would have to infer host capability from scattered string checks
inside selection or command-building code. This workstream should keep that
knowledge local to the inventory/report seam.
