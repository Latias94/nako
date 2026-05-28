# Source-Aware Transcode Runtime - Handoff

Status: Completed
Last updated: 2026-05-28

## Current State

The lane is newly opened to continue after completed playback/transcode policy,
FFmpeg hardware planner, probe inventory, CPU readiness, and startup degradation
workstreams.

SATR-020 through SATR-080 are implemented. Nako now has source-aware stream
technical facts, database round-trip for those facts, an internal
`TranscodeRequirement` emitted by playback decisions, and HLS execution policy
planning that consumes stored ffprobe source streams before selecting hardware
decode/filter/encode stages. HLS FFmpeg command construction is split into
global, device/input, stream map, filter graph, video encoder, audio encoder,
subtitle, and muxer parts. HLS runs now request FFmpeg progress on stdout,
parse bounded runtime metrics, and persist those metrics on transcode sessions.
HLS segment serving now treats running sessions as streamable when a segment is
already present, returns not-ready conflicts for missing in-progress segments,
can wait once according to the transcode throttle setting, and can prune stale
sibling `.ts` segments without deleting the requested segment.

Public playback decisions expose typed reason/report facts while hiding
internal `TranscodeRequirement` and host locators. Admin support evidence exposes
source-aware readiness reasons and redaction-safe runtime metrics, with the
Admin TypeScript contract and admin-web mock response refreshed.

The lane is closed. There is no active task in this workstream.

## Active Task

- None. SATR-010 through SATR-100 are complete.

## Decisions Since Last Update

- FFmpeg CLI remains the first execution adapter.
- rsmpeg is reference pressure for future typed adapter ergonomics, not a
  replacement target in the first execution slice.
- The first code slice starts with source media facts because every later
  decoder, filter, HDR, subtitle, and runtime explanation depends on them.
- There is no external compatibility requirement yet. Prefer deleting shallow
  internal compatibility fields once the source-aware path owns the call chain.
- Public Client DTOs still hide internal transcode requirements and raw
  locators; Admin diagnostics can expose bounded facts later.
- Source-aware pipeline selection currently treats VAAPI, QSV, and VideoToolbox
  as hardware-decode paths for H.264 8-bit input. NVENC and AMF remain software
  decode plus hardware encode in the current command model.
- QSV command planning now emits `-hwaccel qsv` only before `-i`; the encode
  stage owns only `-c:v h264_qsv`.
- Runtime metrics are numeric and redaction-safe: frame count, fps, bitrate,
  total bytes, output time, dup/drop frames, speed, and progress state. They do
  not carry raw paths, command lines, or stderr.
- Progressive HLS serving is file-fact driven: running sessions may serve a
  segment that exists on disk, but a missing running segment is reported as a
  conflict instead of a terminal 404.
- HLS segment cleanup is conservative and only targets stale sibling `.ts`
  files in the session directory; it keeps the requested segment and does not
  remove playlists or sidecar files.

## Blockers

- None.

## Next Recommended Action

- Open a new lane for adaptive HLS ladders/fMP4 or rsmpeg adapter feasibility
  when the next transcode-runtime push starts.
