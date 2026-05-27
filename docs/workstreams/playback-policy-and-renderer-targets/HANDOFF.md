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

PRT-030 is complete. Shared playback permission policy, effective policy,
target kind/network/transport/control vocabulary, and planner-facing
`PlaybackTarget` records now exist. They are not enforced yet.

PRT-040 is complete. The planner consumes `PlaybackTarget` and
`EffectivePlaybackPolicy`, returns internal denied decisions, and Public Client
API maps them to safe `denied` / `policy_denied` wire values. Server playback
currently passes default policy/target values to preserve existing behavior.

## Active Task

- Task ID: PRT-050
- Owner: codex
- Files: `crates/nako-server/src/app/playback`,
  `crates/nako-server/src/http/playback.rs`, `crates/nako-server/src/http/access.rs`
- Validation: `cargo nextest run -p nako-server playback --no-fail-fast`
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
- Core owns shared policy and target vocabulary; `nako-playback` owns the
  planner-facing `PlaybackTarget` because it combines target facts with
  `ClientPlaybackCapabilities`.
- Public Client gets only safe denied mode/reason; detailed policy rows and
  role/access internals remain server/Admin concerns.

## Blockers

- None.

## Next Recommended Action

Start PRT-050 by resolving effective playback policy in the server app from
authenticated principal and Library Access, then enforce denied planner
decisions before creating tickets, Playback Sessions, Transcode Sessions, or
artifacts.
