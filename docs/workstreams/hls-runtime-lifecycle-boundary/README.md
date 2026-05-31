# HLS Runtime Lifecycle Boundary

Status: Closed
Last updated: 2026-05-31

This workstream freezes and deepens the HLS runtime lifecycle boundary before
Nako adds queueing, remote workers, LL-HLS/CMAF, artifact I/O pressure
admission, or richer restart behavior.

The current HLS path works, but lifecycle ownership is spread across playback
composition, HLS app service reservation, transcode runner execution, playlist
readiness checks, segment waits, runtime admission, and cleanup. This lane
starts with a docs/research invariant freeze before any behavior change.

Planner-approved lane: `playback-transcode`.

Current task: none. The workstream closed after `HRLB-040`.

Read before implementation:

- `CONTEXT.md`
- `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/hls-runtime-lifecycle-boundary/CONTEXT.jsonl`

Do not expand this workstream into FFmpeg command planning, transcode
capability inventory, Public/Admin DTO changes, storage schema changes, player
UX, or release packaging without planner approval.

`HRLB-010` froze the lifecycle invariants and test coverage map without Rust
behavior changes. Artifact I/O pressure should split to PAIP rather than being
hidden inside the lifecycle refactor.

`HRLB-020` added behavior-preserving lifecycle tests for timeout cleanup, HLS
startup stale recovery, and remote staged-input release. It did not introduce a
coordinator/facade because the first useful slice was coverage, not a new
abstraction.

`HRLB-030` split follow-ons and recommends
`proposed:hls-progressive-readiness-test-stability` as the next bounded
workstream before PAIP artifact I/O pressure or LL-HLS/CMAF. PAIP, resource
admission unification, remote workers, LL-HLS/CMAF, and player UX remain
separate proposed lanes.

`HRLB-040` first found that the full HLS gate failed under default nextest
concurrency on progressive readiness tests. The remaining test-stability work
was split to `docs/workstreams/hls-progressive-readiness-test-stability/`.
After that follow-on passed, `HRLB-040` reran the final HLS, formatting, JSON,
and diff gates successfully and closed this lifecycle boundary.
