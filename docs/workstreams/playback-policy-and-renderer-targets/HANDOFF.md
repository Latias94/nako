# Playback Policy And Renderer Targets - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The lane is opened as the next backend playback workstream after
`playback-transcode-policy-deepening`. ADR 0039 defines the boundary:
Library Access remains necessary, Playback Permission Policy narrows allowed
playback behavior, and Renderer Target describes where playback will happen.

No implementation has started yet.

## Active Task

- Task ID: PRT-020
- Owner: codex
- Files: `crates/nako-server/src/http/tests/playback.rs`,
  `crates/nako-server/src/app/tests/playback.rs`, `crates/nako-playback/src/lib.rs`
- Validation: `cargo nextest run -p nako-server playback --no-fail-fast`;
  `cargo nextest run -p nako-playback --no-fail-fast`
- Status: NEEDS_CONTEXT
- Review: pending
- Evidence: pending

## Decisions Since Last Update

- Policy and target design is separated from casting protocol implementation.
- Jellyfin behavior is used as feature pressure, not as a model to copy.
- Casting target vocabulary may exist in this lane, but protocol adapters live
  in `casting-renderer-runtime`.

## Blockers

- None.

## Next Recommended Action

Start PRT-020 with characterization tests that prove the current behavior:
`RequiredLibraryAccess::Play` is the only playback gate, and there is no
separate direct/remux/transcode/remote/cast policy yet.
