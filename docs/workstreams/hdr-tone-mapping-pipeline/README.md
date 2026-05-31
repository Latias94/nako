# HDR Tone Mapping Pipeline

Status: Active
Last updated: 2026-05-31

This workstream is intentionally opened as a docs/research lane first. HDR tone
mapping touches playback capability reporting, transcode policy, FFmpeg filter
planning, hardware acceleration, and server HLS behavior. The first task is to
confirm the smallest executable slice before any code changes.

`HTP-010` completed the docs/research scope freeze. `HTP-020` completed the
playback-only color pipeline requirement slice after the planner merged the
accepted `ACDN-020` audio output baseline into this branch.

Planner-approved lane: `playback-transcode`.

`HTP-030` implemented the software-first HLS HDR-to-SDR media-output slice and
is pending planner review/closeout. Hardware tone mapping, device-specific
filter chains, Dolby Vision/HDR10+ dynamic handling, and operator smoke
matrices remain deferred follow-ons.
