# 0046: FFmpeg Probe Inventory

## Status

Accepted.

## Context

ADR 0045 introduced the **Transcode Pipeline Planner**, but the underlying
runtime probe still gets most of its evidence from `ffmpeg -encoders`. That was
enough to break the old encoder-only selection chain, but it is not enough for
Jellyfin-class playback pressure:

- hardware decode support is not implied by hardware encode support;
- `-hwaccel` support is distinct from decoder and encoder names;
- filters determine whether hardware scaling, upload/download, tone mapping,
  overlay, deinterlace, or subtitle burn-in can stay on a hardware path;
- bitstream filters are separate FFmpeg capabilities and will matter for HDR,
  Dolby Vision, AV1/HEVC metadata, and optimized versions;
- Admin diagnostics should report which capability stage is missing without
  exposing raw host paths or full command output.

`repo-ref/jellyfin` applies this pressure by validating and caching decoders,
encoders, hwaccels, filters, and bitstream-filter options before higher-level
encoding helpers make decisions. Nako should keep the same architectural
direction while using Nako's own records and tests.

## Decision

Nako will introduce a structured **FFmpeg Probe Inventory** inside
`nako-transcode`.

The probe Adapter may run multiple FFmpeg discovery commands:

- `ffmpeg -encoders`
- `ffmpeg -decoders`
- `ffmpeg -hwaccels`
- `ffmpeg -filters`
- `ffmpeg -bsfs`

The parsed result is a redaction-safe inventory of capability names grouped by
stage. `HardwareAccelerationReport` is still the public transcode runtime
summary, but its `stage_capabilities` must be populated from the structured
inventory instead of static guesses whenever probe output is available.

The ownership split remains:

- `nako-transcode` owns FFmpeg probe execution, parsing, stage capability
  mapping, and transcode runtime inventory.
- `nako-server` owns startup composition and Admin mapping.
- `nako-api` exposes redaction-safe Admin DTOs only.
- Public Client API remains free of FFmpeg capability details.

The first implementation keeps HLS H.264/AAC as the executable output, but the
probe shape must be able to describe future HDR tone mapping, subtitle burn-in,
adaptive HLS ladders, and optimized-version workflows.

## Consequences

- Hardware capability evidence becomes stage-specific and closer to the real
  host FFmpeg build.
- Pipeline fallback can distinguish "encoder missing" from "decoder/hwaccel/
  filter missing" as planner requirements grow.
- Admin Web can show meaningful hardware diagnostics without shelling out or
  handling raw FFmpeg output.
- The command runner remains separate from the probe parser; command planning
  still consumes typed policy, not raw FFmpeg lists.

## Alternatives Considered

- **Keep deriving stages from encoder names:** rejected because it hides real
  failure causes and makes HDR/subtitle/filter work fragile.
- **Run full smoke transcodes for every stage at startup:** rejected for now
  because startup must stay bounded; smoke tests remain explicit operator
  evidence.
- **Move probe parsing to `nako-server`:** rejected because FFmpeg capability
  semantics belong with transcode execution, not app composition.
- **Adopt Jellyfin's validator directly:** rejected because Nako has its own
  domain records and reference-code rules forbid copying source, schemas,
  comments, or tests.

## Related Workstreams

- `docs/workstreams/ffmpeg-probe-inventory/`
- `docs/workstreams/ffmpeg-hardware-pipeline-planner/`
