# Android Playback Depth Validation - Milestones

Status: Active
Last updated: 2026-05-19

## M0 - Scope Freeze

Exit criteria:

- Direct Play depth target is documented.
- HLS/remux/golden/CI non-goals are explicit.

Evidence:

- `TODO.md` APDV-010 complete
- `DESIGN.md`

## M1 - Playback Advancement

Exit criteria:

- `profile-with-media` smoke captures evidence that playback advanced after
  player launch.
- The check is deterministic enough for local regression.

Evidence:

- focused smoke command
- generated player advancement artifact

## M2 - Server Readback

Exit criteria:

- Smoke reads server **User Playback State** after player exit.
- Evidence proves the server state changed through the public contract.

Evidence:

- focused smoke command
- generated server readback artifact

## M3 - Closeout

Exit criteria:

- Workstream docs reflect shipped behavior.
- Follow-ons are split or explicitly deferred.
- Fresh validation evidence exists.
