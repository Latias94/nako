# Playback Runtime Resource Scheduler — Handoff

Status: Completed
Last updated: 2026-05-29

## Current State

PRRS-010 through PRRS-050 are complete. The lane is closed as a shipped
single-node first slice. It is linked from playback architecture indexes, has a
server-owned playback resource demand and admission decision model, enforces
HLS/remux process permits at start boundaries, and reports runtime pressure
through Admin diagnostics.

The implementation should preserve the current public direct/remux/HLS route
contracts. `nako-transcode` runner semaphores remain low-level execution
guards; this lane adds a host-owned playback admission boundary above them.

## Active Task

None. Use a new workstream for follow-on behavior that changes queueing,
distributed execution, OS isolation, per-device capacity policy, or HLS
artifact I/O enforcement.

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
- PRRS-050 closes this workstream after refreshing architecture docs and
  evidence. The remaining work is deliberately split rather than added to this
  lane.

## Blockers

- None.

## Follow-ons

- `proposed:playback-admission-queueing-and-waitlist` for waiting or priority
  admission instead of immediate busy rejection.
- `proposed:remote-transcode-worker-runtime` for distributed execution and
  worker capacity reporting.
- `proposed:playback-os-resource-isolation` for cgroups, process priority, and
  vendor-specific GPU scheduling policy.
- `proposed:playback-device-capacity-tuning` for per-device and per-host
  capacity calibration.
- `proposed:hls-artifact-io-pressure-enforcement` for disk-sensitive segment
  read/write pressure if operator evidence requires enforcement.
- Existing proposed LL-HLS, DASH/CMAF, and DRM/key-delivery lanes should reuse
  this admission vocabulary instead of adding ad hoc process limits.

## Next Recommended Action

- Commit the PRRS-050 closeout docs after verification.
- Open a named follow-on only when product scope requires one of the deferred
  capabilities above.
