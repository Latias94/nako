# MVP Release Shape

Status: Active
Last updated: 2026-06-01

This workstream defines the first release-shaped Nako MVP before more
Jellyfin/Plex-class breadth is added. It is a planner-owned convergence lane,
not an implementation lane.

The objective is to make future work answer one question first:

```text
Does this unblock the first self-hosted, video-first, single-admin Nako release?
```

## Authoritative Docs

- `MVP.md`: MVP statement, user journey, P0/P1/P2 scope, and non-goals.
- `RELEASE_CUT.md`: initial release cut and known limitations.
- `GAP_MATRIX.md`: release blockers, evidence, and lane routing.
- `DESIGN.md`: planning problem, target state, and architecture direction.
- `TODO.md`: task ledger for turning this scope into an executable release
  convergence plan.
- `EVIDENCE_AND_GATES.md`: source coverage and validation gates.
- `HANDOFF.md`: current continuation state.

## Current State

`MRS-010` opened the workstream and recorded the first MVP shape. The next
task is `MRS-020`: verify the release cut against current code, docs, active
workstreams, and known validation gates.

## Non-Goals

- No Rust or frontend implementation in this workstream.
- No Jellyfin plugin compatibility.
- No built-in tunnel or first-party relay commitment.
- No Addon Manager process supervision.
- No mobile, TV, or desktop-native implementation.
- No new provider breadth unless it blocks the MVP release cut.
