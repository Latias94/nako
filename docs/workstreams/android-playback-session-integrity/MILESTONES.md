# Android Playback Session Integrity - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Scope And Contract Freeze

Exit criteria:

- Workstream docs exist.
- First slice is bounded to session identity and Android launch propagation.
- Direct Play is documented as sessionless.

## M1 - Public Session Identity Bridge

Status: Complete

Exit criteria:

- A public client-safe contract exposes session identity for remux/HLS when a
  session exists.
- Android playback target/session models can carry that identity.
- `NakoBrowseShell` no longer drops a non-Direct session id when launching the
  player.
- Focused Rust and Android unit tests prove the contract.

## M2 - Android Smoke Session Evidence

Status: Complete

Exit criteria:

- Smoke can generate an artifact proving session creation/readback for a
  non-Direct playback path.
- The artifact is token-safe and does not expose local server filesystem paths.
- Structured smoke reports link the artifact.

## M3 - Exit/Cancel Semantics And Closeout

Status: Complete

Exit criteria:

- Active non-ended session exit/cancel behavior is proven, or a narrower
  runtime blocker is split.
- Final gates are fresh.
- Workstream status and handoff accurately describe follow-ons.
