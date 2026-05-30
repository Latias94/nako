# HDR Tone Mapping Pipeline

Status: Draft
Last updated: 2026-05-30

This workstream is intentionally opened as a docs/research lane first. HDR tone
mapping touches playback capability reporting, transcode policy, FFmpeg filter
planning, hardware acceleration, and server HLS behavior. The first task is to
confirm the smallest executable slice before any code changes.

First task: `HTP-010`.

Planner-approved lane: `playback-transcode`.

`HTP-010` is safe to run beside audio compatibility only because it is
docs/research-only. Do not implement HDR code in parallel with `ACDN-020`.
