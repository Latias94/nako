# CPU Transcode Readiness

Status: Complete
Last updated: 2026-05-27

## Why This Lane Exists

The FFmpeg probe inventory now knows which encoders exist, but CPU transcode is
still treated as always available. That makes hardware fallback too optimistic:
if the host FFmpeg lacks `libx264` or `aac`, HLS fallback to CPU will plan
successfully and fail later at command execution.

## Relevant Authority

- ADRs:
  - `docs/adr/0045-ffmpeg-hardware-pipeline-planner.md`
  - `docs/adr/0046-ffmpeg-probe-inventory.md`
  - `docs/adr/0047-cpu-transcode-readiness.md`
- Workstreams:
  - `docs/workstreams/ffmpeg-probe-inventory/`

## Target State

For probe-derived reports, CPU HLS readiness is based on required software
output encoders:

- `libx264` for video;
- `aac` for audio.

Pipeline planning rejects explicit CPU HLS planning when CPU readiness is
missing. Hardware fallback-to-CPU also rejects when the requested hardware path
and the CPU fallback path are both unavailable.

## In Scope

- CPU capability mapping from `FfmpegProbeInventory`.
- New pipeline readiness reasons for software pipeline unavailability and CPU
  fallback unavailability.
- Admin readiness reason mapping.
- Fake FFmpeg test scripts that represent software encoder availability.
- Focused transcode/API/server playback tests.

## Out Of Scope

- Full source-codec decoder matrix.
- FFmpeg muxer/protocol probing.
- Server startup degradation model for missing FFmpeg.
- Frontend diagnostics rendering.
- Remote workers.

## Closeout Condition

This lane can close when:

- probe-derived CPU capability is unavailable when `libx264` or `aac` is
  missing;
- pipeline planning does not fall back to an unavailable CPU path;
- Admin diagnostics expose typed software/fallback unavailability;
- focused gates pass and follow-ons are explicit.

## Closeout Summary

Completed on 2026-05-27.

The shipped boundary keeps CPU transcode readiness inside `nako-transcode`.
Probe-derived reports now require `libx264` and `aac` for the current HLS
H.264/AAC software pipeline. Pipeline planning rejects both explicit CPU
selection and hardware fallback-to-CPU when that software path is unavailable.
Admin diagnostics expose typed readiness reasons while the capability list keeps
the missing encoder evidence bounded and redacted.

Follow-ons remain intentionally separate:

- source-codec-aware decoder requirements;
- muxer/protocol probing for HLS outputs;
- startup degradation so browse/direct-play can remain available when HLS
  transcode is unavailable.
