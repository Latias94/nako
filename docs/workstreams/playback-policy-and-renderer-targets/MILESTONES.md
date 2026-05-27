# Playback Policy And Renderer Targets - Milestones

Status: Completed
Last updated: 2026-05-27

## M0 - Workstream Open

Exit criteria:

- ADR 0039 exists and states the policy/target boundary.
- Workstream docs agree on scope, non-goals, and task order.
- Jellyfin-class reference pressure is captured as architecture pressure, not
  copied implementation.

Primary evidence:

- `docs/adr/0039-playback-policy-and-renderer-target-boundary.md`
- `docs/workstreams/playback-policy-and-renderer-targets/DESIGN.md`
- `docs/workstreams/playback-policy-and-renderer-targets/TODO.md`

## M1 - Current Behavior Characterization

Exit criteria:

- Tests prove the current Library Access-only gate.
- Tests identify where direct/remux/HLS can currently start without
  mode-specific policy.
- Existing compatible playback behavior remains green.

Primary gates:

- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo nextest run -p nako-playback --no-fail-fast`

## M2 - Policy And Target Domain Records

Exit criteria:

- Policy records model direct/remux/audio-transcode/video-transcode/remote/cast
  permissions.
- Renderer target records model target kind, network scope, transport auth, and
  control capability.
- Defaults preserve current admin/local behavior unless a later task changes
  them explicitly.

Primary gates:

- `cargo nextest run -p nako-core playback --no-fail-fast`
- `cargo nextest run -p nako-playback --no-fail-fast`

## M3 - Planner Enforcement

Exit criteria:

- Planner consumes effective policy and target capabilities.
- Denial reasons are typed and safe for Public Client mapping.
- Planner remains pure and repository-free.

Primary gates:

- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-api public --no-fail-fast`

## M4 - Server App And HTTP Integration

Exit criteria:

- Server app resolves effective playback policy before playback starts.
- Denied direct/remux/HLS paths do not create Playback Sessions, Transcode
  Sessions, artifacts, or tickets.
- Routes stay thin adapters.

Primary gates:

- `cargo nextest run -p nako-server playback --no-fail-fast`

## M5 - API And Diagnostics

Exit criteria:

- Public Client API exposes safe target/capability and denial vocabulary.
- Admin API exposes effective-policy readiness/diagnostics without leaking raw
  policy rows or secrets.
- Generated contracts are refreshed when API shape changes.

Primary gates:

- `cargo nextest run -p nako-client-protocol public --no-fail-fast`
- `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast`

## M6 - Closeout

Exit criteria:

- All task evidence is recorded.
- `WORKSTREAM.json` status and completed tasks are current.
- Casting handoff is updated with concrete assumptions from this lane.
- Follow-on work is split instead of keeping this lane open.

Primary gates:

- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

Closeout result:

- Completed on 2026-05-27.
- `casting-renderer-runtime` is the active follow-on.
