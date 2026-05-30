# Audio Compatibility Downmix Normalization

Status: Active
Last updated: 2026-05-30

This workstream makes audio compatibility an explicit playback/transcode
contract. The first target is to model **Audio Output Requirement** in
`nako-playback`, then propagate it into transcode policy and FFmpeg planning
without mixing it with HDR tone mapping, subtitle burn-in, or web player work.

First executable task: `ACDN-020`.

Planner-approved lane: `playback-transcode`.

Read before implementation:

- `CONTEXT.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/audio-compatibility-downmix-normalization/CONTEXT.jsonl`

Do not edit `nako-transcode` in `ACDN-020`; that work starts at `ACDN-030`.
