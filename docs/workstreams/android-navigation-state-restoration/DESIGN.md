# Android Navigation State Restoration

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

`docs/workstreams/android-route-back-stack-refactor/` closed the single-route
overwrite problem by introducing `NakoBrowseNavigationState` and
`NakoRouteStack`. That state is still held with `remember`, so Activity
recreation drops the user's current browse context.

This is now the next clean architecture step: make the navigation module
responsible for its own restoration policy instead of letting `NakoBrowseShell`
silently reset to Home.

## Relevant Authority

- `CONTEXT.md`: **Client Applications**, **Media Item**, **Genre**, **Tag**,
  **People**, and **Public Client API** terminology.
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/workstreams/android-route-back-stack-refactor/`

## Problem

The app has a proper navigation model, but its lifetime is too short. A
configuration change or process restoration should not discard safe browse
context such as:

- selected top-level destination;
- Media Item detail route;
- Genre, Tag, or Person facet route;
- Server Profile route opened from Settings.

At the same time, Player routes contain `PlaybackLaunchRequest`, including a
real request with headers. Persisting that route would couple restoration to
runtime playback state and risk storing sensitive request material.

## Target State

When this lane closes:

- `NakoBrowseShell` uses `rememberSaveable` for browse navigation state.
- The save/restore format is explicit, versionable, and covered by focused JVM
  tests.
- Safe routes restore: Top Level, Item Detail, Browse Facet, and Server Profile.
- Player routes are treated as transient and restore to the previous safe route.
- Invalid or stale saved navigation data restores to a safe root state.
- No playback request URL, token, or header is persisted in saved navigation
  state.

## In Scope

- Android browse navigation state and saver code under
  `apps/android/app/src/main/java/dev/nako/android/ui/browse`.
- Focused JVM tests under
  `apps/android/app/src/test/java/dev/nako/android/ui/browse`.
- Workstream evidence and closeout docs.

## Out Of Scope

- Jetpack Navigation adoption.
- Deep links or route argument URI contracts.
- Restoring active playback sessions.
- Persisting playback source picker decisions.
- UI redesign or smoke harness changes unless a regression is found.

## Architecture Direction

Keep `NakoBrowseNavigationState` as the caller-facing module and add a dedicated
saveable adapter at the Compose seam. The saved form should be a small structured
snapshot with only safe route arguments. Player should never be serialized; the
restore policy should drop it and keep the previous safe route when available.

This keeps restoration local, testable, and reversible. Jetpack Navigation can
still be adopted later if deep links, back-stack inspection, or route argument
contracts become large enough to justify it.

## Closeout Condition

This lane can close when:

- save/restore code and tests are implemented;
- `NakoBrowseShell` uses the saver;
- focused route restoration tests pass;
- Android compile or unit gate passes;
- docs record the transient Player policy and remaining follow-ons.

## Implemented Outcome

Closed on 2026-05-19 with `NakoBrowseNavigationStateSaver`. The saved payload
is a JSON snapshot containing only the selected top-level destination and safe
route arguments. It restores Top Level, Item Detail, Browse Facet, and Server
Profile routes. Player routes remain transient and restore to the previous safe
route, which avoids persisting playback request URLs or authorization headers.

`NakoBrowseShell` now uses `rememberSaveable` with this saver for browse
navigation state.
