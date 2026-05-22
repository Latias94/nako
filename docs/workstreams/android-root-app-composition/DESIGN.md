# Android Root App Composition

Status: Closed
Last updated: 2026-05-20

## Problem

`NakoAndroidApp` still owns the Android client dependency graph directly:
transport, profile store, token vault, connection client, browse client,
playback client, playback preferences, user playback client, device playback
position store, app runtime, and app session.

That makes the root UI function a shallow composition module. Future app-wide
concerns such as auth/session refresh, dynamic color policy, preference
composition, telemetry, or test-time dependency replacement would require
editing the root Composable instead of one composition seam.

## Target State

- Introduce a root app composition module that creates and names the Android
  runtime dependencies.
- Keep `NakoAppSession` as the root app state machine.
- Keep `NakoAndroidAppContent` as a rendering shell over root environment and
  app session state.
- Preserve the current connection/browse/player behavior.
- Add focused tests for root app runtime/session creation behavior that can be
  verified without Compose.

## Scope

- `apps/android/app/src/main/java/dev/nako/android/ui/NakoAndroidApp.kt`
- New root app composition module under `apps/android/app/src/main/java/dev/nako/android/ui/`
- Focused root composition tests under `apps/android/app/src/test/java/dev/nako/android/ui/`
- Workstream docs under this directory.

## Non-Goals

- Do not add auth/session refresh behavior in this lane.
- Do not change server API contracts.
- Do not redesign Material UI.
- Do not replace existing client classes or stores.
- Do not introduce a dependency injection framework.

## Architecture Direction

Use a plain Kotlin composition object, not a DI framework. The module should be
deep enough to name and own root dependencies, but not abstract concrete
classes merely for test doubles when existing interfaces already exist.

The interface should be small: callers should know how to build a root
environment and how to create a `NakoAppSession`, not how every client is wired.

## Closeout Notes

The lane closed with `NakoAppEnvironment` and
`AndroidNakoAppEnvironmentFactory` owning the root dependency graph.
`NakoAndroidApp` now builds one environment and one `NakoAppSession`, then
passes them to `NakoAndroidAppContent`.

`NakoAndroidAppContent` is internal and no longer accepts every client/store as
separate parameters. Root UI rendering still chooses between connection and
browse modes from `NakoAppSession.state`.

The Android factory uses `applicationContext` for platform-backed stores and
vaults.
