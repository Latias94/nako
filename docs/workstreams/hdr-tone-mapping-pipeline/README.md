# HDR Tone Mapping Pipeline

Status: Draft
Last updated: 2026-05-30

This workstream is intentionally opened as a docs/research lane first. HDR tone
mapping touches playback capability reporting, transcode policy, FFmpeg filter
planning, hardware acceleration, and server HLS behavior. The first task is to
confirm the smallest executable slice before any code changes.

`HTP-010` completed the docs/research scope freeze. The next implementation
task is `HTP-020`, but it remains blocked while `ACDN-020` is active on the
shared playback vocabulary files.

Planner-approved lane: `playback-transcode`.

`HTP-010` is safe to run beside audio compatibility only because it is
docs/research-only. Do not implement HDR code in parallel with `ACDN-020`.
