# Android Playback Start Flow Coordinator - Design

Status: Closed
Last updated: 2026-05-19

## Problem

`NakoBrowseShell` currently owns too much playback start behavior. It reacts to
source picker events, resolves resume position, starts Public Client session
preflight, constructs `PlaybackLaunchRequest`, maps errors, and opens the
player route. That made the active-remux fix correct but left business flow
inside a Compose shell.

The shell should remain responsible for UI state and navigation. Playback start
semantics should be testable without rendering the shell.

## Target State

- A small Android playback start coordinator/use case owns playback start
  preflight, resume resolution, and launch request construction.
- The coordinator is covered by focused JVM tests using public Android client
  types.
- `NakoBrowseShell` delegates start behavior to the coordinator and keeps only
  state transitions plus route navigation.
- Active-remux smoke semantics remain unchanged: source checking does not start
  a server session, and player start does.

## Architecture Direction

- Keep this Android-side only. Do not change server contracts.
- Use existing `PlaybackPreferencesStore`, `NakoPlaybackClient`,
  `DevicePlaybackPositionStore`, and `PlaybackLaunchRequest` models.
- Prefer a narrow coordinator API over adding more mutable state to
  `NakoBrowseShell`.
- Preserve token safety: failures return existing `SafePlaybackDiagnostics`.

## Non-Goals

- Redesign source picker UI.
- Change playback decision scoring or server route selection.
- Add new smoke states.
- Rework navigation stack persistence.

## Assumptions

- The active-remux cancellation lane already proves the end-to-end runtime
  behavior; this lane is a maintainability refactor around that behavior.

## Outcome

- Added `PlaybackStartCoordinator` as the playback start boundary.
- Moved resume resolution from `ui.browse` into `player`.
- `NakoBrowseShell` now delegates playback start semantics and only handles UI
  state plus route navigation.
