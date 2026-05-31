# HDR Tone Mapping Pipeline

Status: Closed
Last updated: 2026-05-31

This workstream shipped the first HDR tone-mapping slice for Nako playback:
playback-owned color pipeline requirements and transcode-owned software-first
HLS HDR-to-SDR command planning.

`HTP-010` completed the docs/research scope freeze. `HTP-020` completed the
playback-only color pipeline requirement slice after the planner merged the
accepted `ACDN-020` audio output baseline into this branch.

Planner-approved lane: `playback-transcode`.

`HTP-030` implemented the software-first HLS HDR-to-SDR media-output slice and
planner verification accepted it into `main`. Hardware tone mapping,
device-specific filter chains, Dolby Vision/HDR10+ dynamic handling, and
operator smoke matrices remain deferred follow-ons.
