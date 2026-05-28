# Playback Runtime Boundary Deepening - Milestones

Status: Completed
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Non-goals prevent feature creep into adaptive/fMP4/rsmpeg work.
- First executable slice is HLS artifact serving.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`

## M1 - HLS Artifact Serving Boundary

Exit criteria:

- HLS artifact serving logic is local to a focused module/service.
- `PlaybackAppService` delegates playlist/segment artifact mechanics.
- Existing HLS route behavior is unchanged.

Primary gate:

- `cargo nextest run -p nako-server hls --no-fail-fast`

## M2 - Support Evidence And Diagnostics Boundary

Exit criteria:

- Support evidence/runtime diagnostics collection is either extracted or has an
  evidence-backed no-split decision.
- Redaction contracts remain unchanged.

Primary gate:

- `cargo nextest run -p nako-server admin_v1_playback --no-fail-fast`

## M3 - Store Port And Test Locality Audit

Exit criteria:

- Store trait narrowing is performed only where it reduces real coupling.
- Tests prove behavior at the new boundaries.

Primary gate:

- `cargo nextest run -p nako-server playback --no-fail-fast`

## M4 - Closeout

Exit criteria:

- Fresh gates are recorded.
- Workstream docs reflect shipped boundaries.
- Follow-ons are split or explicitly deferred.
- `WORKSTREAM.json` status is updated.

Status: Complete on 2026-05-28.
