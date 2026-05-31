# Playback Compatibility Matrix Hardening

Status: Active
Last updated: 2026-05-31

This workstream deepens `nako-playback` with a table-driven compatibility
matrix for Direct Play, Remux, and HLS Transcode decisions. It is intentionally
crate-local so it can run in parallel with HDR `HTP-030` without touching
`nako-transcode` or `nako-server`.

Planner-approved lane: `playback-transcode`.

First executable task: `PCMH-020`.

Read before implementation:

- `CONTEXT.md`
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
- `docs/adr/0044-playback-capability-profile-planner.md`
- `docs/architecture/PLAYBACK.md`
- `docs/workstreams/playback-compatibility-matrix-hardening/CONTEXT.jsonl`

Do not expand this workstream into transcode command planning, server HLS
composition, Public Client DTO changes, device profile databases, or web player
behavior.
