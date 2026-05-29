# HLS Selected Main Audio Cleanup - Milestones

Status: Completed
Last updated: 2026-05-29

## M0 - Scope And Evidence Freeze

Status: Completed

Exit criteria:

- Problem and target state are explicit.
- Related HLS audio sidecar workstreams are linked.
- Non-goals are explicit.
- First executable task is chosen.

Primary evidence:

- `docs/workstreams/hls-selected-main-audio-cleanup/DESIGN.md`
- `docs/workstreams/hls-selected-main-audio-cleanup/TODO.md`

## M1 - Duplication Characterization

Status: Completed

Exit criteria:

- Tests explain the current selected-main-audio duplication.
- Single-audio and no-audio expectations are protected.
- The implementation surface for removing duplication is clear.

Primary gates:

- `cargo nextest run -p nako-transcode hls --no-fail-fast`
- `cargo nextest run -p nako-server hls --no-fail-fast`

## M2 - Main Mux Cleanup

Status: Completed

Exit criteria:

- Sidecar-capable multi-audio HLS main outputs avoid selected audio duplication.
- Generated audio sidecars remain manifest-backed and advertised through
  `TYPE=AUDIO`.
- Single-audio and no-sidecar behavior stays stable.

Primary gates:

- `cargo nextest run -p nako-transcode hls --no-fail-fast`
- `cargo nextest run -p nako-server hls --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`

## M3 - Closeout

Status: Completed

Exit criteria:

- Fresh focused gate evidence is recorded.
- Architecture docs reflect the shipped output shape.
- Follow-ons are split or explicitly deferred.
- `WORKSTREAM.json` status is updated.

Primary evidence:

- `docs/workstreams/hls-selected-main-audio-cleanup/CLOSEOUT.md`
- `docs/workstreams/hls-selected-main-audio-cleanup/EVIDENCE_AND_GATES.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
