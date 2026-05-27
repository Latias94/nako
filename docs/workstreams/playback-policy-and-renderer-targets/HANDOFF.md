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
was still passing default policy/target values at that point.

PRT-050 is complete. Playback Permission Policy now has persisted user/role
rows behind `PlaybackPolicyRepository`. The server app resolves effective
playback policy from the authenticated user and Library Access before issuing
browser tickets or starting direct/remux/HLS playback. Denied direct playback
does not create a Playback Session; denied remux/HLS playback does not create
Playback Sessions, Transcode Sessions, or artifacts.

## Active Task

- Task ID: PRT-060
- Owner: codex
- Files: `crates/nako-client-protocol/src`, `crates/nako-api/src`,
  `apps/admin-web/src/adminApi/generated/contract.ts`
- Validation: `cargo nextest run -p nako-client-protocol public --no-fail-fast`;
  `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast`
- Status: READY
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
- First storage slice persists playback policy rows because app-service policy
  resolution needed normal repository-backed tests instead of route-local
  hard-coding.
- Multiple matching role playback policies are merged restrictively; a user
  playback policy overrides role playback policy for the same library.
- Administrators keep administrator playback defaults; Library Access remains
  the first playback gate.

## Blockers

- None.

## Next Recommended Action

Start PRT-060 by adding safe Public/Admin DTOs for target/capability and
effective playback policy diagnostics. Do not expose raw policy rows, role
assignment internals, source locators, FFmpeg command strings, or ticket
secrets through Public Client responses.
