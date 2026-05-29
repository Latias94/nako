# Playback Runtime Resource Scheduler — Handoff

Status: Active
Last updated: 2026-05-29

## Current State

PRRS-010 and PRRS-020 are complete. The lane is open, linked from playback
architecture indexes, and now has a server-owned playback resource demand and
admission decision model before enforcing permits in HLS/remux start paths.

The implementation should preserve the current public direct/remux/HLS route
contracts. `nako-transcode` runner semaphores remain low-level execution
guards; this lane adds a host-owned playback admission boundary above them.

## Active Task

- Task ID: PRRS-030
- Owner: unassigned
- Files:
  - `crates/nako-server/src/app/playback/mod.rs`
  - `crates/nako-server/src/app/playback/hls.rs`
  - `crates/nako-server/src/app/playback/remux.rs`
  - `crates/nako-server/src/app/tests/playback.rs`
  - `crates/nako-server/src/http/tests/playback.rs`
- Validation:
  - `cargo nextest run -p nako-server hls --no-fail-fast`
  - `cargo nextest run -p nako-server playback --no-fail-fast`
- Status: PENDING
- Review: pending
- Evidence: `docs/workstreams/playback-runtime-resource-scheduler/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Keep this lane single-node first.
- Do not add remote workers, LL-HLS, DASH, DRM/key delivery, OS cgroups, or
  durable queueing in the first proof.
- Model playback resource demand in `nako-server`, because admission needs user
  policy, session, storage, and route context.
- Keep FFmpeg command planning and low-level runner permits in `nako-transcode`.
- Reuse paths must not double-acquire process permits for already-running
  sessions.
- PRRS-020 models direct remote streams as host-owned accepted/rejected classes,
  remux process work as low-level runner guarded, HLS CPU/GPU transcode work as
  low-level runner guarded, and HLS artifact I/O as not-yet-enforced.

## Blockers

- None for PRRS-030.

## Next Recommended Action

- Run `run-workstream-task` for PRRS-030.
- Wire admission permit acquisition into HLS/remux start paths without
  double-acquiring for reused sessions.
