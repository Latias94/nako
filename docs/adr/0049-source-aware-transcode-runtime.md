# 0049: Source-Aware Transcode Runtime

## Status

Proposed.

## Context

Nako has completed the first playback/transcode architecture deepening wave:

- ADR 0038 split playback planning and transcode policy seams.
- ADR 0044 made playback capability planning profile-driven.
- ADR 0045 introduced a stage-aware FFmpeg hardware pipeline planner.
- ADR 0046 added structured FFmpeg probe inventory.
- ADR 0047 made CPU HLS readiness depend on real software encoder probes.
- ADR 0048 made unavailable HLS transcode a runtime capability instead of a
  startup invariant.

Those decisions make the current HLS/remux path safe, but not yet deep enough
for a self-hosted media server that must handle varied source files. The
remaining weakness is source awareness. Nako currently cannot reliably explain
or plan around codec profile, level, bit depth, pixel format, color transfer,
HDR metadata, frame rate, rotation, subtitle delivery, attached fonts, audio
shape, or per-source hardware decode limitations.

Jellyfin shows the product pressure: mature media servers validate decoders,
encoders, filters, hwaccels, bitstream filters, devices, tone mapping,
subtitles, throttling, segment cleanup, and transcode job progress. FFmpeg
documentation shows that correct hardware use is device and filter-graph
specific. rsmpeg shows a possible future typed FFmpeg API direction, but Nako's
current executable adapter is still FFmpeg CLI.

Reference code is used only for architecture and behavior pressure. Nako must
not copy Jellyfin or FFmpeg implementation code, schemas, tests, comments, or
assets.

## Decision

Nako will deepen the transcode runtime through a source-aware flow:

```text
MediaProbeResult
  -> PlaybackPlanner
  -> TranscodeRequirement
  -> TranscodePipelinePlanner
  -> FfmpegCommandPlan
  -> TranscodeJobRuntime
```

The ownership split is:

- `nako-core` owns stable media technical facts and durable transcode session
  progress records.
- `nako-media-probe` owns ffprobe execution and mapping into those facts.
- `nako-playback` owns playback compatibility, selected streams, explicit
  transcode reasons, and structured `TranscodeRequirement` records.
- `nako-transcode` owns source-aware pipeline planning, fallback decisions, and
  FFmpeg command planning.
- `nako-server` owns runtime orchestration, cancellation, progress persistence,
  HLS serving, throttling, cleanup, and Admin diagnostic adaptation.
- `nako-api` exposes stable wire DTOs and redaction-safe evidence.

The first executable output may remain HLS H.264/AAC while the internal model
deepens. The important change is that the model must carry source facts and
requirements before command generation. HDR tone mapping, subtitle burn-in,
adaptive ladders, and progressive HLS can then be added as vertical slices
without turning `ffmpeg.rs` into a monolithic decision helper.

FFmpeg command construction should be split by concern: device initialization,
input decode, filter graph, video encoder, audio encoder, subtitle handling,
and muxer output. The builder consumes already-planned requirements and
pipeline stages; it does not own playback policy.

rsmpeg may be evaluated later as a second execution adapter or a typed
inspection layer. It is not required before the CLI path becomes source-aware.

## Consequences

- Playback decisions can explain why a source needs transcode with more precise
  reasons than unsupported container or codec.
- Hardware fallback can depend on source codec/profile/bit-depth/HDR facts
  instead of static encoder availability alone.
- HDR, subtitle, and HLS output features get explicit planning inputs before
  command branches are added.
- Admin diagnostics can distinguish static runtime readiness, per-source
  planning gaps, and live transcode job progress.
- Public Client API remains redaction-safe and does not expose raw host paths,
  device paths, or command strings.
- Some schema or JSON compatibility work may be required as media facts and
  transcode progress become durable.

## Alternatives Considered

- **Keep adding options to `FfmpegCommandBuilder::hls`:** rejected because it
  would mix playback policy, source analysis, hardware fallback, filter graph
  decisions, and command argument ordering in one adapter.
- **Replace FFmpeg CLI with rsmpeg first:** rejected for this lane because the
  current product gap is planning/runtime depth, not the process API shape.
- **Copy Jellyfin's encoding helper model:** rejected because Nako has Rust
  crate boundaries and repository rules forbid copying reference
  implementation details.
- **Delay media probe facts until HDR/subtitle features are implemented:**
  rejected because later planner and command tests need stable source facts.

## Related Workstreams

- `docs/workstreams/source-aware-transcode-runtime/`
- `docs/workstreams/playback-transcode-policy-deepening/`
- `docs/workstreams/ffmpeg-hardware-pipeline-planner/`
- `docs/workstreams/ffmpeg-probe-inventory/`
- `docs/workstreams/cpu-transcode-readiness/`
- `docs/workstreams/playback-transcode-startup-degradation/`
