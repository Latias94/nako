# 0047: CPU Transcode Readiness

## Status

Accepted.

## Context

ADR 0046 made FFmpeg probe inventory stage-aware, but the pipeline still treats
CPU transcode as always available. That is not true for the executable HLS path:
software HLS currently requires at least a video encoder for H.264 output and an
audio encoder for AAC output.

The incorrect assumption matters because CPU is both an explicit acceleration
choice and the fallback target for hardware failure. If `libx264` or `aac` is
missing, a hardware fallback-to-CPU decision is not actually executable.

## Decision

Nako will model CPU HLS transcode readiness from FFmpeg probe inventory.

For the current single-variant HLS H.264/AAC output, `nako-transcode` treats
these software encode capabilities as required CPU stages:

- `libx264` for H.264 video output;
- `aac` for AAC audio output.

`HardwareAccelerationReport` remains the planner input. The CPU capability in a
report built from `FfmpegProbeInventory` must be unavailable when required
software encode capabilities are missing. Static test reports may still use
`HardwareAccelerationReport::cpu_only()` when the test deliberately bypasses
FFmpeg probing.

Pipeline planning must not silently fall back to CPU when the CPU pipeline is
unavailable. Admin diagnostics should expose this as a typed readiness reason
without exposing raw FFmpeg output.

## Consequences

- CPU fallback becomes a real executable capability instead of a policy wish.
- Hardware fallback behavior can distinguish "GPU unavailable but CPU fallback
  ready" from "GPU unavailable and CPU fallback unavailable".
- Tests and fake FFmpeg scripts must include software encoders when they expect
  CPU fallback or default CPU transcode readiness.
- Future work should add source-codec-aware decoder requirements once
  `TranscodePipelineRequest` carries input codec metadata.

## Alternatives Considered

- **Keep CPU always available:** rejected because it hides a runtime failure
  behind a successful planner decision.
- **Require all possible software decoders now:** rejected because the planner
  does not yet know the input codec set.
- **Fail server startup on global FFmpeg probe failure:** deferred. Startup
  readiness and transcode service degradation need their own lane so direct play
  and browse workflows are not coupled to HLS availability.

## Related Workstreams

- `docs/workstreams/cpu-transcode-readiness/`
- `docs/workstreams/ffmpeg-probe-inventory/`
