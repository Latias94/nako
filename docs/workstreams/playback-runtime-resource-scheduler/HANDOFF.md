# Playback Runtime Resource Scheduler — Handoff

Status: Active
Last updated: 2026-05-29

## Current State

PRRS-010, PRRS-020, PRRS-030, and PRRS-040 are complete. The lane is open,
linked from playback architecture indexes, has a server-owned playback resource
demand and admission decision model, enforces HLS/remux process permits at start
boundaries, and reports runtime pressure through Admin diagnostics.

The implementation should preserve the current public direct/remux/HLS route
contracts. `nako-transcode` runner semaphores remain low-level execution
guards; this lane adds a host-owned playback admission boundary above them.

## Active Task

- Task ID: PRRS-050
- Owner: planner
- Files:
  - `docs/workstreams/playback-runtime-resource-scheduler`
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Validation:
  - `cargo nextest run -p nako-server playback --no-fail-fast`
  - `cargo nextest run -p nako-server hls --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
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
- PRRS-040 adds Admin `resource_pressure` diagnostics for configured capacity,
  available permits, in-use permits, resource class, and enforcement mode.
- Admin runtime pressure diagnostics are redaction-safe and generated TypeScript
  contracts are refreshed for both web surfaces.

## Blockers

- None for PRRS-050.

## Next Recommended Action

- Run `close-workstream` or `run-workstream-task` for PRRS-050.
- Verify closeout gates, update architecture maps if needed, and either close
  the lane or split queueing, remote workers, OS isolation, and per-device
  tuning into named follow-ons.
