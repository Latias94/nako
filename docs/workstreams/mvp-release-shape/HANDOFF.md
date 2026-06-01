# MVP Release Shape - Handoff

Status: Active
Last updated: 2026-06-01
Current task: `MRS-050`

## Current State

The MVP release convergence lane is open. `MRS-010` created the initial
release shape, `MRS-020` verified it against current repo evidence, `MRS-030`
converted it into an MVP validation ladder, and `MRS-040` aligned the active
queue:

- `MVP.md` defines a video-first, self-hosted, single-admin release.
- `RELEASE_CUT.md` records the initial P0/P1/P2 cut.
- `GAP_MATRIX.md` now classifies P0 blockers, evidence-backed P0 areas,
  conditional rows, and deferred P1/P2 rows.
- `WORKSTREAM.json` registers lane slug `mvp-release-convergence`.
- Architecture and roadmap links point to this workstream.
- `EVIDENCE_AND_GATES.md` now records required, conditional, and gap-routing
  release gates.
- `CAMPAIGNS.md` records the parallel campaign split, integration status, and
  remaining optional/conditional campaigns.
- Campaign A (`PTJCH-220`) and Campaign B (`web-mvp-live-smoke`) have been
  merged to `main` after `integrate-lane-results` review and post-merge gates.

Important `MRS-020` findings:

- Install/release, scan/source state, metadata authority, storage health, Admin
  diagnostics, Addon Sidecar foundation, and documented remote access are
  evidence-backed enough for MVP planning.
- Browser/web remains the accepted MVP client path. The deterministic
  `web-mvp-live-smoke` gate now covers the Web/Public Client browse/detail/
  playback journey.
- `PTJCH-220` is no longer a P0 blocker after merge. `PTJCH-310` artifact I/O
  pressure is a follow-on unless release-candidate gates escalate it.
- `GAMA-060` is conditional/P1 unless MVP requires Web Generated Artifact
  apply. `CSAPA-050` is P1/deferred if browser/web remains the first client.
- GAMA/CSAPA active-lane drift remains visible but is not MVP-blocking.

## Next Action

Continue `MRS-050` only for release-candidate coordination:

Key checks:

- run the documented MVP validation ladder from a clean release-candidate
  worktree;
- split an `operations-release` gate-wrapper task only if the team wants one
  command for the full video-first ladder;
- keep `GAMA-060` and `CSAPA-050` out of the MVP campaign unless the product
  cut changes.

## Stop Conditions

Return to planner/user decision before:

- changing Rust, frontend, schema, public API, or generated contracts;
- changing accepted ADR decisions;
- making `nako-official-addons` changes;
- promoting a P1/P2 capability into P0 without evidence;
- treating reference code from Jellyfin or Plex-like projects as copyable
  implementation material.

## Suggested Goal

```text
Set the current Codex goal to execute the MVP release-candidate validation ladder from docs/workstreams/mvp-release-shape/EVIDENCE_AND_GATES.md. Keep PTJCH-220 and web-mvp-live-smoke as integrated evidence, split an operations-release wrapper only if one-command proof is required, and do not mutate GAMA/CSAPA active ledgers from this lane.
```
