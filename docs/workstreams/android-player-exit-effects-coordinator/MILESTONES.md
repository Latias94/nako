# Android Player Exit Effects Coordinator - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Lane Setup

Exit: APEC-010 complete.

## M1 - Exit Coordinator Extraction

Exit:

- Player exit side-effect wiring is callable from a non-Compose coordinator.
- `PlaybackPlayerRoute` no longer passes user playback and cancellation lambdas.
- Existing local persistence, progress/watched reporting, and cancellation
  semantics are preserved by focused JVM tests.

Status: Complete.

## M2 - Closeout

Exit:

- Focused JVM tests and diff checks pass.
- Workstream docs record evidence and residual follow-ons.

Status: Complete.
