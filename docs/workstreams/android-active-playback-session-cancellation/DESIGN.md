# Android Active Playback Session Cancellation - Design

Status: Closed
Last updated: 2026-05-19

## Problem

The previous Android playback session integrity lane proves that Public Client
API exposes remux/HLS session identity and that Android exit code requests
session cancellation for unfinished playback. It does not prove the runtime
case where the player exits while the server-side remux/HLS job is still
running.

The gap exists because the current smoke path plays `profile-with-media` through
Direct Play, and the existing remux preflight waits for ffmpeg completion before
returning a session id. A smoke script can create and read back a remux session,
but it cannot currently make the Android player hold an active non-terminal
session and then cancel it from player exit.

## Target State

- Android smoke has a dedicated fixture state for active remux playback
  cancellation.
- The fixture forces a non-Direct playback decision without changing normal
  product defaults.
- Android receives a Public Client session id before server-side remux finishes.
- Exiting the Android player requests cancellation through
  `/playback/sessions/{session_id}/cancel`.
- Smoke evidence reads the same session back through Public Client API and
  proves a terminal `cancelled` state with `cancelled` failure category.

## Architecture Direction

- Keep the authority on Public Client API boundaries. Android must not use
  admin diagnostics, local server paths, or server logs as truth.
- Add an explicit active-session preparation path for remux before using it in
  smoke. A synchronous remux stream response cannot prove active cancellation.
- Use debug-only Android fixture seed data to force smoke playback capabilities.
  Normal Android users should continue to use the default Direct Play preference.
- Keep smoke artifacts token-safe and free of local filesystem paths.

## Non-Goals

- Full adaptive HLS live playback semantics.
- UI redesign of player or source picker.
- Admin API diagnostics for Android.
- Compatibility with outdated Android playback code.

## Closed Decisions

- Start with remux cancellation because server tests already have a slow remux
  fixture and cancel route coverage.
- HLS active cancellation can reuse the same pattern later if runtime playlist
  readiness becomes asynchronous.
- Android source checking now builds a client-safe playback target without
  starting a server-side session. Session preflight happens only when the user
  starts playback, so long-running remux jobs are not created during source
  picker evidence capture.
