# Playback Policy And Renderer Targets - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The lane is opened as the next backend playback workstream after
`playback-transcode-policy-deepening`. ADR 0039 defines the boundary:
Library Access remains necessary, Playback Permission Policy narrows allowed
playback behavior, and Renderer Target describes where playback will happen.

PRT-020 characterization is complete. Current behavior is now fixed in tests:
Library Access `Play` is the only playback gate, browser tickets can be issued
for direct/remux/HLS with no mode-specific policy, app remux has no principal
or policy input, and remote context is not a permission gate.

## Active Task

- Task ID: PRT-030
- Owner: codex
- Files: `crates/nako-core/src`, `crates/nako-playback/src`
- Validation: `cargo nextest run -p nako-core playback --no-fail-fast`;
  `cargo nextest run -p nako-playback --no-fail-fast`
- Status: NEEDS_CONTEXT
- Review: pending
- Evidence: pending

## Decisions Since Last Update

- Policy and target design is separated from casting protocol implementation.
- Jellyfin behavior is used as feature pressure, not as a model to copy.
- Casting target vocabulary may exist in this lane, but protocol adapters live
  in `casting-renderer-runtime`.
- Characterization confirmed the current gap is real: no per-user
  direct/remux/transcode/remote/cast playback policy exists yet.

## Blockers

- None.

## Next Recommended Action

Start PRT-030 by adding pure playback permission policy and renderer target
records. Keep policy resolution out of `nako-playback`; the planner should
receive an already-resolved effective policy in PRT-040.
