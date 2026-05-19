# Android Active Playback Session Cancellation - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Lane Setup

Exit: APSC-010 complete.
Status: Complete.

## M1 - Active Remux Session Contract

Exit:

- Public Client remux preflight/start can expose a session id while the remux
  job is still non-terminal.
- Public cancel route can cancel that active session.
- Focused server test proves the lifecycle without admin APIs.
Status: Complete.

## M2 - Android Smoke Fixture Path

Exit:

- Dedicated smoke state seeds a debug-only playback capability override.
- Source picker shows Remux route prepared.
- Player opens with a real session id and exits before completion.
- Public Client session readback proves `cancelled`.
Status: Complete.

## M3 - Closeout

Exit:

- Workstream docs and Android smoke docs describe the shipped behavior.
- Fresh evidence is recorded.
- Remaining work, if any, is split with a clear boundary.
Status: Complete. HLS active cancellation remains a separate future fixture,
not unfinished remux scope.
