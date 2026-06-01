# Architecture Roadmap Reconciliation - Handoff

Status: Active
Last updated: 2026-06-01

## Current State

This lane is open to reconcile roadmap and architecture navigation after the
latest sub-architecture audit and GAMA closeout.

Six read-only audit agents inspected Playback/Transcode, Storage/VFS,
Library/Metadata/NFO/Artwork, Web/Product, State/Access, and
Control/Addons/Ops/Realtime. All reported `DONE_WITH_CONCERNS`: current
implementation is generally ahead of some roadmap/status docs, with no
blocking code issue discovered by the audit.

## Active Task

- Task ID: `ARR-050`
- Lane: `architecture-planning`
- Status: active
- Owner: planner

Goal: verify docs gates, update final evidence, and decide whether to close
this lane or split broad historical cleanup into a follow-on.

## Key Files

- `docs/GOALS.md`
- `docs/ROADMAP.md`
- `docs/workstreams/README.md`
- `docs/architecture/LANES.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/STATE_ACCESS.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/STORAGE_VFS.md`

## Decisions

- Open a short-lived planner/docs lane before new parallel implementation
  work, because queue state is currently the main risk.
- Keep implementation follow-ons proposed until this reconciliation validates
  the next queue.
- Do not perform broad historical handoff cleanup unless the stale reference
  can misroute current work.

## Completed In This Lane

- `ARR-020`: program-level roadmap and active queue docs now route active work
  to this planner lane.
- `ARR-030`: capability maps and workstream links now reflect shipped provider,
  playback policy, artwork, Web, addon, realtime, and control-plane evidence.
- `ARR-040`: high-risk stale references that could misroute future work were
  repaired, including the stale ADR 0053 path and WMLP/PBSI heartbeat handoff.

## Blockers

- None.

## Follow-Ons To Consider After Closeout

- `proposed:generated-artifact-bulk-metadata-apply`
- `proposed:generated-artifact-provider-mapping-breadth`
- `proposed:metadata-provider-depth-and-precision`
- `proposed:admin-settings-api-backed-restoration`
- `proposed:hls-artifact-io-pressure-enforcement`
- `proposed:playback-admission-queueing-and-waitlist`
- `proposed:vfs-cache-repair-diagnostics`
- `proposed:library-watcher-and-media-intake-stability`
- `proposed:durable-job-priority-policy-and-scheduler-migration`
- `proposed:control-plane-observability-and-trace-context`
- `proposed:self-hosted-remote-access-and-endpoint-discovery`
