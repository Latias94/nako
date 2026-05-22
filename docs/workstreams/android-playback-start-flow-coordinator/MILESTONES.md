# Android Playback Start Flow Coordinator - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Lane Setup

Exit: APSF-010 complete.

## M1 - Coordinator Extraction

Exit:

- Playback start flow is callable from a non-Compose coordinator/use case.
- Coordinator handles session-backed start preflight only at start time.
- Coordinator returns `PlaybackLaunchRequest` on success and
  `SafePlaybackDiagnostics` on failure.
- `NakoBrowseShell` delegates to the coordinator.

Status: Complete.

## M2 - Closeout

Exit:

- Focused JVM tests and diff checks pass.
- Workstream docs record evidence and residual follow-ons.

Status: Complete.
