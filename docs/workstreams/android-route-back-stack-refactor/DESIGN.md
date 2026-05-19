# Android Route Back-Stack Refactor

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

The Android browse shell currently stores one `TaruRoute`. Opening a detail
facet, player, or Server Profile replaces the previous route, so every Back
callback returns to Top Level instead of the page the user came from.

This became visible in the detail facet smoke lane: after opening a Genre, Tag,
or Person facet from detail, smoke had to reopen the Media Item from Home
because Back discarded the detail route.

## Relevant Authority

- `CONTEXT.md`: **Client Applications**, **Media Item**, **Genre**, **Tag**,
  **People**, and **Public Client API** terminology.
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/workstreams/android-detail-facet-smoke-evidence/`

## Problem

Single-route navigation is a shallow module: callers must know the return
target and encode it at each screen boundary. That spreads workflow rules across
`TaruBrowseShell` callbacks:

- Detail Back returns Top Level.
- Facet Back returns Top Level even when opened from Detail.
- Player Back returns Top Level even when opened from Detail.
- Server Profile Back returns Top Level even when opened from Settings.

The implementation also makes smoke scripts compensate for app navigation
instead of testing the natural workflow.

## Target State

When this lane closes:

- `TaruBrowseShell` owns a small navigation state module with top-level
  destination selection, route push, route pop, and root clearing behavior.
- Back behavior is context-preserving:
  - Home -> Detail -> Facet -> Back -> Detail
  - Home -> Detail -> Player -> Back -> Detail
  - Settings -> Server Profile -> Back -> Settings
- Top Level remains the root and bottom navigation remains visible only at root.
- Existing browse, detail, facet, player, settings, and smoke flows still pass.
- Focused unit tests cover the back-stack rules without Compose or emulator.
- Smoke evidence covers at least Detail -> Facet -> Back -> Detail.

## In Scope

- Android route model and browse shell navigation state under
  `apps/android/app/src/main/java/dev/taru/android/ui/browse`.
- Focused JVM tests for route stack behavior.
- `Smoke-Emulator.ps1` navigation simplification and return-path assertion.
- Android README / smoke fixture docs if behavior or evidence changes.

## Out Of Scope

- Jetpack Navigation adoption.
- Deep links, saved-state restoration across process death, or multi-window
  navigation.
- Server, Public Client API, or playback runtime behavior.
- UI redesign beyond behavior required for clean back-stack navigation.

## Architecture Direction

Introduce an Android-local navigation state model that keeps root Top Level and
top-level destination selection separate from nested routes. The interface
should stay small:

- read current route;
- push a route;
- pop to the previous route, falling back to Top Level at root;
- clear to Top Level when bottom navigation destinations change.
- retain the selected top-level destination when a nested route is popped.

This is intentionally not Jetpack Navigation yet. The current shell is still
small, and a focused route stack gives the app correct behavior without pulling
in route serialization, argument encoding, or saved-state policy before those
needs exist.

## Implemented Outcome

Closed on 2026-05-19 with `TaruBrowseNavigationState` and `TaruRouteStack`.
`TaruBrowseShell` now uses `open`, `navigateBack`, and `selectDestination`
instead of assigning route return targets inside each screen callback. Android
system Back is handled through `BackHandler` whenever the route stack can pop.

Smoke now proves:

- Settings -> Server Profile -> Back -> Settings.
- Home -> Detail -> Facet -> Back -> Detail.
- Home -> Detail -> Player -> Back -> Detail.

## Closeout Condition

This lane can close when:

- route stack model and tests are implemented;
- browse shell uses the model for Detail, Facet, Player, Server Profile, and
  Top Level transitions;
- smoke verifies Detail -> Facet -> Back -> Detail;
- focused Android tests and smoke/regression gates pass;
- docs record the remaining navigation follow-ons.
