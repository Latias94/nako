# HDR Tone Mapping Pipeline

Status: Active
Last updated: 2026-05-30

This workstream is intentionally opened as a docs/research lane first. HDR tone
mapping touches playback capability reporting, transcode policy, FFmpeg filter
planning, hardware acceleration, and server HLS behavior. The first task is to
confirm the smallest executable slice before any code changes.

`HTP-010` completed the docs/research scope freeze. The planner merged the accepted `ACDN-020` audio output baseline into this branch, so `HTP-020` is ready as a playback-only implementation slice.

Planner-approved lane: `playback-transcode`.

`HTP-020` may run beside `ACDN-030` only because it is playback-only. Do not edit transcode, server HLS, Public Client API DTOs, media probe schemas, or web player code in `HTP-020`.
