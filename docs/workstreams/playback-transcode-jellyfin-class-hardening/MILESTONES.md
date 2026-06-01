# Playback Transcode Jellyfin-Class Hardening - Milestones

Status: Closed
Last updated: 2026-06-01

## M0 - Interface Freeze

Exit criteria:

- Seam map is recorded in `DESIGN.md`.
- Owned scopes and shared scopes are recorded in `TODO.md`.
- Stop conditions are recorded before implementation starts.
- Architecture links point to this workstream.

## M1 - First Parallel Batch Ready

Exit criteria:

- Worker prompts and branch/worktree guidance exist for Playback Capability,
  Transcode Pipeline Capability, and FFmpeg Adapter lanes.
- Each prompt lists owned scopes, forbidden scopes, dependencies, and gates.
- The first three implementation tasks can run without editing the same primary
  ownership surface.

## M2 - First Parallel Batch Implemented Or Split

Exit criteria:

- `PTJCH-110`, `PTJCH-120`, and `PTJCH-130` are complete, or each has been
  split into a dedicated workstream with clear ownership.
- Tests prove playback planning, pipeline capability, and FFmpeg adapter
  behavior remain separated.

## M3 - HLS Runtime And Artifact Coordination

Exit criteria:

- HLS Artifact Authority and Playback Runtime ownership are either implemented
  in coordinated tasks or split into separate follow-ons.
- Request identity, artifact manifest, session lifecycle, and admission
  behavior have one clear owner each.

## M4 - Artifact I/O Decision And Closeout

Exit criteria:

- Artifact I/O pressure is explicitly accepted into this workstream or split to
  a PAIP follow-on.
- Workstream state, evidence, and handoff are current.
- Final validation gates pass or residual risks are recorded with follow-up
  scope.

Status: Complete. Artifact I/O pressure is split to
`proposed:hls-artifact-io-pressure-enforcement`; PTJCH is closed with final
docs/json/diff gates and no additional Rust changes.
