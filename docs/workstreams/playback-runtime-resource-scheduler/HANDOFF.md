# Playback Runtime Resource Scheduler — Handoff

Status: Active
Last updated: 2026-05-29

## Current State

PRRS-010, PRRS-020, and PRRS-030 are complete. The lane is open, linked from
playback architecture indexes, has a server-owned playback resource demand and
admission decision model, and now enforces HLS/remux process permits at start
boundaries.

The implementation should preserve the current public direct/remux/HLS route
contracts. `nako-transcode` runner semaphores remain low-level execution
guards; this lane adds a host-owned playback admission boundary above them.

## Active Task

- Task ID: PRRS-040
- Owner: unassigned
- Files:
  - `crates/nako-server/src/app.rs`
  - `crates/nako-server/src/http/admin.rs`
  - `crates/nako-server/src/http/tests/system.rs`
  - `crates/nako-api/src/admin/playback.rs`
- Validation:
  - `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast`
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
  remux process work as admission-permit guarded, HLS CPU/GPU transcode work as
  admission-permit guarded, and HLS artifact I/O as not-yet-enforced.
- PRRS-030 keeps permit lifetime with the runtime task. Browser playback
  preflight paths pre-acquire permits before spawning and transfer ownership to
  the background HLS/remux run so immediate route returns do not release
  capacity early.
- Reuse of active or completed HLS/remux sessions remains outside new permit
  acquisition.

## Blockers

- None for PRRS-040.

## Next Recommended Action

- Run `run-workstream-task` for PRRS-040.
- Surface configured capacity, available permits, and current pressure in Admin
  playback runtime diagnostics without exposing local paths or command lines.
