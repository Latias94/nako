# Playback Compatibility Matrix Hardening

Status: Closed
Last updated: 2026-05-31

This workstream deepened `nako-playback` with a table-driven compatibility
matrix for Direct Play, Remux, and HLS Transcode decisions. It stayed
crate-local and did not touch `nako-transcode`, `nako-server`, Public Client
DTOs, persisted preferences, device profile databases, or player behavior.

Planner-approved lane: `playback-transcode`.

Shipped task: `PCMH-020`.

Read before follow-on work:

- `CONTEXT.md`
- `docs/adr/0038-playback-planning-and-transcode-policy-seams.md`
- `docs/adr/0044-playback-capability-profile-planner.md`
- `docs/architecture/PLAYBACK.md`
- `docs/workstreams/playback-compatibility-matrix-hardening/CONTEXT.jsonl`

Open a new workstream before expanding into full device profile matrices,
transcode command planning, server HLS composition, Public Client DTO changes,
device profile databases, or web player behavior.
