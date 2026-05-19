# Android Device-Local Playback Position

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

Android already records playback position through a route-scoped
`DevicePlaybackPositionStore`, but the default app composition uses an in-memory
store. That means the "Local resume" behavior works only inside the current app
process. A phone user expects a device-local resume point to survive normal app
restarts, while Taru must still avoid claiming server-authoritative or
cross-device **User Playback State**.

## Relevant Authority

- ADR:
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- Existing workstreams:
  - `docs/workstreams/android-client-foundation/`
  - `docs/workstreams/android-material-expressive-ui/`
  - `docs/workstreams/android-developer-validation-entrypoint/`
- Domain glossary:
  - `CONTEXT.md`: **User Playback State** is distinct from local client state.

## Problem

The current in-memory Android playback position store loses device-local resume
state when the process is killed. The UI already labels the behavior as
device-local, so the next correct step is persistence scoped to the active
server profile, Media Item, and Media Source.

## Target State

When this lane closes:

- Android persists device-local playback position across app process restarts.
- Position keys remain scoped by server profile id, Media Item id, and Media
  Source id.
- Ended playback and non-positive positions clear the stored resume point.
- The app still presents this as local-only state and does not claim
  cross-device Continue Watching or server-authoritative **User Playback State**.
- Tests prove persistence, scoping, clearing, and corrupted local data handling.

## In Scope

- Android local persistence under `apps/android`.
- Unit tests for the store contract.
- App composition change from in-memory to persistent store.
- Workstream evidence and Android local validation.

## Out Of Scope

- Public Client API changes.
- Server-side **User Playback State**.
- Cross-device Continue Watching.
- Playback progress reporting to Taru.
- Session-id envelope changes for remux/HLS.
- Downloads/offline playback.

## Architecture Direction

Keep the existing `DevicePlaybackPositionStore` contract. Add a
SharedPreferences-backed implementation that serializes only local playback
position records, using a key derived from the existing scoped
`DevicePlaybackPositionKey`.

This persistence is client-owned local state. It should live beside current
Android player models, not in server profile storage and not in the token vault.
The in-memory store remains useful for tests and previews.

## Closeout

Closed on 2026-05-19. Android now uses a SharedPreferences-backed
`DevicePlaybackPositionStore` in app composition while keeping the in-memory
store for tests and previews.

Final evidence:

- Focused unit gate:
  `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackLaunchTest --no-daemon`
- Local validation report:
  `apps/android/build/validation/20260519-100247/report.md`
- Diff hygiene:
  `git diff --check`

## Closeout Condition

This lane can close when:

- the persistent store is implemented and wired into `TaruAndroidApp`;
- Android unit tests cover persistence, scoping, clear behavior, and corrupt
  data fallback;
- `Validate-AndroidLocal.ps1 -SkipSmoke` passes;
- `git diff --check` passes;
- follow-ons for server-authoritative state and session envelopes remain split.
