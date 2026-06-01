# MVP Release Shape

Status: Closed
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
- `CAMPAIGNS.md`: campaign split and integration ledger.
- `DESIGN.md`: planning problem, target state, and architecture direction.
- `TODO.md`: task ledger for turning this scope into an executable release
  convergence plan.
- `EVIDENCE_AND_GATES.md`: source coverage and validation gates.
- `CLOSEOUT.md`: closeout decision, gates, follow-ons, and residual risks.
- `HANDOFF.md`: current continuation state.

## Closed State

`MRS-010` opened the workstream and recorded the first MVP shape. `MRS-020`
verified the release cut, `MRS-030` defined the validation ladder, `MRS-040`
aligned active queue risk, and `MRS-050` launched the first implementation
campaigns and validated the release candidate ladder.

Campaign A (`PTJCH-220`) and Campaign B (`web-mvp-live-smoke`) are now
integrated on `main`. Gate 0 through Gate 5 and Gate 7 pass; Gate 6 is skipped
by MVP scope because this candidate does not claim an official Addon Sidecar
proof. Future release execution should open a new operations/release workstream
instead of reopening this planning lane.

## Non-Goals

- No Rust or frontend implementation in this workstream.
- No Jellyfin plugin compatibility.
- No built-in tunnel or first-party relay commitment.
- No Addon Manager process supervision.
- No mobile, TV, or desktop-native implementation.
- No new provider breadth unless it blocks the MVP release cut.
