# HLS Runtime Lifecycle Boundary

Status: Active
Last updated: 2026-05-31

This workstream freezes and deepens the HLS runtime lifecycle boundary before
Nako adds queueing, remote workers, LL-HLS/CMAF, artifact I/O pressure
admission, or richer restart behavior.

The current HLS path works, but lifecycle ownership is spread across playback
composition, HLS app service reservation, transcode runner execution, playlist
readiness checks, segment waits, runtime admission, and cleanup. This lane
starts with a docs/research invariant freeze before any behavior change.

Planner-approved lane: `playback-transcode`.

Current task: `HRLB-010`.

Read before implementation:

- `CONTEXT.md`
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/hls-runtime-lifecycle-boundary/CONTEXT.jsonl`

Do not expand this workstream into FFmpeg command planning, transcode
capability inventory, Public/Admin DTO changes, storage schema changes, player
UX, or release packaging without planner approval.
