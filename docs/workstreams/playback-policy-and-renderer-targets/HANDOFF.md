# Playback Policy And Renderer Targets - Handoff

Status: Completed
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

PRT-060 is complete. Public playback decisions now carry safe target facts and
policy denial outcomes. Admin playback runtime/support diagnostics now expose
policy readiness, not raw user/role policy rows. Public TypeScript/Kotlin SDKs
and the Admin Web generated contract were refreshed.

PRT-070 is complete. Fresh closeout gates passed, review found no blocking
findings, and the follow-on casting lane is unblocked.

## Active Task

None. This workstream is closed.

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
- Public playback target/denial DTOs are safe outcome DTOs. They must not grow
  raw policy rows, role assignment internals, source locators, ticket secrets,
  or FFmpeg command details.
- Admin policy diagnostics describe policy readiness and resolution rules. They
  intentionally do not expose stored user/role policy row contents.
- Closeout keeps persistent policy editing, bitrate-limit enforcement, and
  target-kind request input as separate follow-ons instead of extending this
  boundary-focused lane.

## Blockers

- None for this closed workstream.
- Frontend note: `npm --prefix apps/admin-web run check` still fails on
  existing Playback Sessions page/mock-data type drift unrelated to PRT-060's
  policy/target DTO additions.

## Next Recommended Action

Start `casting-renderer-runtime` CAST-020. The next backend slice should
characterize the missing Renderer Session/control surface before adding the
renderer domain and Nako-to-Nako adapter.
