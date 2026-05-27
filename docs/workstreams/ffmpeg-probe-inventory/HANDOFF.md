# FFmpeg Probe Inventory - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

ADR 0046 records that `nako-transcode` owns FFmpeg probe execution, parsing,
and stage capability mapping. The lane shipped `FfmpegProbeInventory`, detector
execution for encoders/decoders/hwaccels/filters/bitstream filters, required vs
optional stage capability diagnostics, and FFmpeg input-option ordering for HLS
hardware decode arguments.

## Active Task

- None. This lane is closed.

## Decisions

- The probe inventory is redaction-safe names only.
- Filter option probing is a follow-on, not part of this first inventory lane.
- HLS H.264/AAC remains the only executable output in this lane.
- Jellyfin is reference pressure only.

## Blockers

- None.

## Next Action

Recommended follow-ons:

- Add FFmpeg filter-option probing for tone mapping and overlay options before
  implementing HDR/subtitle burn-in command generation.
- Add software codec readiness for CPU fallback instead of treating CPU HLS as
  always available.
- Tighten QSV/VideoToolbox decoder command generation once input codec metadata
  is part of `TranscodePipelineRequest`.
- Connect Admin Web to required/optional stage capability diagnostics when the
  frontend lane is ready.
