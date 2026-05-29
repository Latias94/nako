# Playback Runtime Resource Scheduler — Handoff

Status: Active
Last updated: 2026-05-29

## Current State

PRRS-010 is complete. The lane is open and linked from playback architecture
indexes. The first implementation slice is PRRS-020: introduce a server-owned
playback resource demand and admission decision model before enforcing permits
in HLS/remux start paths.

The implementation should preserve the current public direct/remux/HLS route
contracts. `nako-transcode` runner semaphores remain low-level execution
guards; this lane adds a host-owned playback admission boundary above them.

## Active Task

- Task ID: PRRS-020
- Owner: unassigned
- Files:
  - `crates/nako-server/src/app/playback`
  - `crates/nako-server/src/app/tests/playback.rs`
  - `crates/nako-server/src/config.rs`
- Validation:
  - `cargo nextest run -p nako-server playback_resource --no-fail-fast`
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

## Blockers

- None for PRRS-020.

## Next Recommended Action

- Run `run-workstream-task` for PRRS-020.
- Add the smallest typed admission model and tests before wiring enforcement
  into HLS/remux start paths.

