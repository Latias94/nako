# Android Browse Shell Session Host

Status: Closed
Last updated: 2026-05-20

## Problem

`NakoBrowseShell` has already moved browse navigation, loading, playback start,
connection, and settings state into dedicated sessions. The remaining shell
still owns session construction, saveable-state synchronization, startup load
effects, route-displayed effects, and settings runtime wiring.

That leaves two forms of architectural friction:

- Compose still knows too much about lifecycle orchestration instead of only
  rendering state and forwarding events.
- Saveable browse state is written only after synchronous dispatch calls. Async
  follow-up state changes, including route changes after playback start, can
  miss the saveable snapshot.

## Target State

Introduce a small, testable browse shell host:

- `BrowseShellHost` owns `BrowseSession`, settings dispatch, lifecycle hooks,
  and saveable-state publishing.
- `BrowseShellRuntime` builds the concrete data source, playback starter,
  resume resolver, and settings runtime for the active profile.
- `NakoBrowseShell` creates adapters with `remember`, collects host state, and
  renders the existing screens.
- Saveable state updates follow `BrowseSession.state`, not just immediate
  dispatch return values.

## Scope

- Android browse shell and tests under `apps/android/app/src/main/java/dev/nako/android/ui/browse`.
- Settings action wiring where it is currently assembled by browse shell.
- Workstream docs under this directory.

## Non-Goals

- Do not adopt Jetpack Navigation.
- Do not redesign browse UI screens.
- Do not change public server API contracts.
- Do not change player engine internals.
- Do not reopen closed route-stack or navigation-restoration behavior unless
  this work exposes a regression.

## Architecture Direction

`BrowseSession` remains the browse state machine. `BrowseShellHost` is a host
module around it: it concentrates lifecycle side effects and integration
adapters behind a small interface. The Compose shell stays shallow and
declarative by depending on host state and dispatch functions.

## Closeout Notes

The lane closed with `BrowseShellHost` owning browse startup, route-displayed
loading, settings action forwarding, saveable-state publishing, and host-scope
cancellation. `ClientBrowseShellRuntime` now centralizes client-backed data
source, playback starter, resume resolver, and settings runtime assembly.

`NakoBrowseShell` no longer constructs `BrowseSession` or `SettingsSession`
directly and no longer uses Compose `LaunchedEffect` for browse route loading.
Callback lambdas are read through `rememberUpdatedState` so parent recomposition
does not rebuild the host just because callback instances change.
