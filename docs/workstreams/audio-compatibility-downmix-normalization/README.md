# Audio Compatibility Downmix Normalization

Status: Completed
Last updated: 2026-05-31

This workstream made audio compatibility an explicit playback/transcode
contract. `nako-playback` now owns **Audio Output Requirement** vocabulary,
`nako-transcode` carries that requirement through policy/profile/pipeline
planning, and FFmpeg HLS command planning emits deterministic downmix and
normalization filters when requested.

Active task: none. `ACDN-010` through `ACDN-050` are complete.

Planner-approved lane: `playback-transcode`.

Read before implementation:

- `CONTEXT.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/audio-compatibility-downmix-normalization/CONTEXT.jsonl`

Follow-ons should be opened as separate workstreams for persisted preferences,
client UI controls, device profile databases, dialogue clarity, subtitle
burn-in, or HDR tone mapping.
