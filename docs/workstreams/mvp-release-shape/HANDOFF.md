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
- `CAMPAIGNS.md` records the current parallel campaign split and worker
  policies.

No Rust or frontend implementation was changed by this lane.

Important `MRS-020` findings:

- Install/release, scan/source state, metadata authority, storage health, Admin
  diagnostics, Addon Sidecar foundation, and documented remote access are
  evidence-backed enough for MVP planning.
- Browser/web remains the accepted MVP client path. It needs a fresh live
  browse/detail/playback smoke gate; split `web-product` work only if existing
  tests plus manual browser smoke are not reproducible enough for release.
- `PTJCH-220` is a P0 blocker until playback runtime ownership and diagnostics
  are finished, split, or explicitly accepted. A parallel playback worker is
  running on this task and should return through `integrate-lane-results`.
- `GAMA-060` is conditional/P1 unless MVP requires Web Generated Artifact
  apply. `CSAPA-050` is P1/deferred if browser/web remains the first client.
- GAMA/CSAPA active-lane drift remains visible but is not MVP-blocking.

## Next Action

Run `MRS-050`: close this planning lane or split focused MVP implementation
campaigns.

Key checks:

- wait for or later integrate the `PTJCH-220` worker result;
- split a small `web-product` live MVP smoke workstream from `CAMPAIGNS.md`
  once planner docs are committed or synced into a clean worktree;
- split an `operations-release` gate-wrapper task if the team wants one command
  for the full video-first ladder;
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
Set the current Codex goal to execute MRS-050 for docs/workstreams/mvp-release-shape. Close the planning lane or split focused MVP campaigns with exact owners, worktrees, gates, and stop conditions. Keep PTJCH-220 as the active P0 blocker until its worker result is integrated. Do not mutate GAMA/CSAPA active ledgers from this lane.
```
