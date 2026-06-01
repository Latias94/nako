# Architecture Roadmap Reconciliation - TODO

Status: Closed
Last updated: 2026-06-01

## M0 - Open Planner Lane

- [x] ARR-010 [owner=planner] [deps=none] [scope=docs/workstreams/architecture-roadmap-reconciliation]
  Goal: Open the reconciliation workstream and record the sub-architecture
  audit input.
  Validation: `python -m json.tool docs/workstreams/architecture-roadmap-reconciliation/WORKSTREAM.json`.
  Evidence: `RECON.md`, `DESIGN.md`, `WORKSTREAM.json`.
  Handoff: Continue at `ARR-020`.

## M1 - Program Roadmap And Queue

- [x] ARR-020 [owner=planner] [deps=ARR-010] [scope=docs/GOALS.md,docs/ROADMAP.md,docs/architecture/LANES.md,docs/workstreams/README.md]
  Goal: Make the top-level goal map, roadmap, active queue, and high-traffic
  workstream navigation reflect the current active planner lane and closed
  recent lanes.
  Validation: `git diff --check -- docs/GOALS.md docs/ROADMAP.md docs/architecture/LANES.md docs/workstreams/README.md docs/workstreams/architecture-roadmap-reconciliation`.
  Evidence: updated planner docs route active work to this lane only.
  Handoff: Continue at `ARR-030`.

## M2 - Architecture Capability Maps

- [x] ARR-030 [owner=planner] [deps=ARR-020] [scope=docs/architecture/WORKSTREAM_LINKS.md,docs/architecture/LIBRARY_PIPELINE.md,docs/architecture/STATE_ACCESS.md,docs/architecture/CONTROL_PLANE.md,docs/architecture/STORAGE_VFS.md]
  Goal: Correct high-risk capability status drift and add missing evidence
  links for shipped provider, playback policy, artwork, Web, addon, realtime,
  and control-plane work.
  Validation: `git diff --check -- docs/architecture docs/workstreams/architecture-roadmap-reconciliation`.
  Evidence: proposed lanes no longer point at already-shipped MVP provider
  slices; partial statuses narrow remaining work.
  Handoff: Continue at `ARR-040`.

## M3 - Targeted Historical Reference Repair

- [x] ARR-040 [owner=planner] [deps=ARR-030] [scope=docs/workstreams]
  Goal: Fix only stale historical references that can misroute future work,
  such as nonexistent ADR paths or handoffs that contradict later shipped
  contracts.
  Validation: targeted `rg` checks listed in `EVIDENCE_AND_GATES.md`.
  Evidence: stale high-risk strings removed or explicitly split to follow-ons.
  Handoff: Continue at `ARR-050`.

## M4 - Verify And Close Or Split

- [x] ARR-050 [owner=planner] [deps=ARR-040] [scope=docs/workstreams/architecture-roadmap-reconciliation]
  Goal: Verify docs checks, update evidence, and either close this lane or
  split broad historical cleanup into a separate follow-on.
  Validation: all gates in `EVIDENCE_AND_GATES.md`.
  Evidence: `HANDOFF.md`, `WORKSTREAM.json`, and `CLOSEOUT.md`.
  Handoff: DONE. Select the next implementation lane from the updated proposed
  queue.
