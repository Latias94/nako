# MVP Release Shape - Handoff

Status: Closed
Last updated: 2026-06-01
Current task: none

## Current State

The MVP release convergence lane is closed. `MRS-010` created the initial
release shape, `MRS-020` verified it against current repo evidence, `MRS-030`
converted it into an MVP validation ladder, `MRS-040` aligned the active queue,
and `MRS-050` integrated the P0 campaign slices and validated the release
candidate ladder:

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
- Release-candidate validation has run on `main`. Gate 0 planner/docs
  preflight, Gate 1 `scripts/release-gate.ps1 -Mode fast`, Gate 2 focused
  server MVP journey tests, Gate 3 Web/Public Client validation, Gate 5 package
  and container shape, and Gate 7 PostgreSQL contract harness now pass. Gate 4
  playback runtime evidence is covered by the Gate 2 `playback`/`hls` filters
  and the post-merge PTJCH seek gate. Gate 6 is skipped by MVP scope because
  this candidate does not claim an official Addon Sidecar proof.
- The Gate 2 retry fixed a stale official external acquisition runner catalog
  assertion. The catalog resolve response now correctly exposes the optional
  `transmission_password` Secret Reference declaration while still proving no
  raw links, bearer tokens, addon tokens, or host internals leak.

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

This workstream should not continue routine execution. Open a focused follow-on
workstream for release execution or new scope.

Follow-on triggers:

- run the official addon alpha smoke only if the candidate scope changes to
  claim an official Addon Sidecar proof;
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
Set a new Codex goal only for a focused follow-on, such as actual release
artifact publication, an operations-release one-command gate wrapper, official
addon alpha smoke, or a product decision that changes the MVP scope. Keep
GAMA/CSAPA active ledgers owned by their lanes.
```
