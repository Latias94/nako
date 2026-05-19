# Android Playback Depth Validation

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

Android smoke now proves the server-backed browse, detail, source picker, and
player-safe launch path. That is enough to catch route, fixture, and UI
regressions, but it does not yet prove that playback actually advances or that
leaving the player reports server-owned **User Playback State** in a way the
server can read back.

This lane deepens playback validation from "the player surface opened" to "the
first Direct Play path really played, exited cleanly, and reported progress
through the public contract."

## Target State

When this lane closes:

- Android smoke has a repeatable Direct Play depth check.
- The check proves playback position advances beyond the initial resume point.
- Exiting the player writes progress or watched state through the Public Client
  API.
- Smoke evidence includes a server readback of **User Playback State** after
  returning from the player.
- Existing smoke reports remain token-safe and locator-safe.
- HLS/remux/session cancellation depth is explicitly deferred or split.

## In Scope

- Direct Play playback advancement evidence for the `Night Harbor` fixture.
- Player exit behavior evidence tied to server **User Playback State**.
- Smoke fixture/readback changes needed to prove the behavior.
- Documentation of the new gate and evidence paths.

## Out Of Scope

- HLS playback validation.
- Remux/transcode playback validation.
- Full playback quality, audio/video sync, subtitle, chapter, or PiP checks.
- Golden screenshot comparison.
- CI/device-farm packaging.
- Changing player product UX beyond what is needed for deterministic evidence.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The existing `profile-with-media` fixture is enough for a first Direct Play depth check. | High | It already opens `Night Harbor` and uses a short direct-play MP4. | Create a separate fixture state with a longer media file. |
| A short fixture can prove position advancement but may not prove long-form watched threshold policy. | High | The demo video is intentionally tiny for smoke speed. | Keep watched-threshold depth as a follow-on with a longer fixture. |
| The server readback should remain the authority for progress evidence. | High | User Playback State workstream closed with server-authoritative semantics. | Android-local player state could hide reporting failures. |

## Architecture Direction

Treat this as a validation-depth lane, not a player rewrite. The smoke script
should keep delegating server state to Public Client API routes and should only
add enough player observation to prove that playback moved and that exit
reporting reached the server.

The first implementation should prefer user-facing or server-facing signals
already available in the app and fixture. If a new signal is required, keep it
debug-only or evidence-only and document why it does not change the production
contract.

This lane is closed. `profile-with-media` now proves Direct Play advancement
to the ended state and records server **User Playback State** readback after
player exit.
