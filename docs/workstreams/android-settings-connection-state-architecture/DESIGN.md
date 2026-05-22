# Android Settings Connection State Architecture - Design

Status: Closed
Last updated: 2026-05-20

## Why This Lane Exists

Browse and Player now use testable session/adapters instead of keeping state
and async orchestration directly inside Composables. Connection setup and
settings are the remaining early-client surfaces with substantial local state:

- `NakoAndroidAppContent` decides whether the connection flow is visible.
- `NakoConnectionShellContent` owns form fields, async connection checks,
  result state, profile persistence, token persistence, active profile
  switching, and failure recording.
- `ServerProfileScreen` owns active profile switching and sign-out side effects.

That works, but it creates a second state-management style inside the Android
client. The clean architecture target is now clear enough to remove that debt.

## Relevant Authority

- ADRs:
  - `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
  - `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- Existing docs:
  - `docs/workstreams/android-unidirectional-state-architecture/`
  - `docs/workstreams/android-presentation-runtime-adapters/`
  - `docs/workstreams/android-player-session-architecture/`
  - `docs/workstreams/android-material-expressive-ui/`

## Problem

Connection and settings state currently has weak locality:

- connection form state and async connection check behavior are mixed with UI;
- profile save/switch behavior is repeated across connection setup, app root,
  and settings;
- sign-out deletes a token directly from a visual screen;
- the app root stores `showConnection` separately from profile state, making
  connection visibility an implicit UI flag rather than an explicit state
  transition.

This makes future multi-server, token refresh, settings persistence, and visual
evidence harder to extend safely.

## Target State

- Connection setup has a `ConnectionSession`:
  - immutable `ConnectionState`,
  - explicit `ConnectionAction`,
  - async connection testing behind a small runtime interface,
  - token/profile persistence behind a small runtime interface.
- Settings/server profile management has a `SettingsSession`:
  - explicit actions for switch profile, sign out, and diagnostics copy data,
  - token deletion and snapshot mutation outside visual rendering.
- Root app state derives connection visibility from an explicit session or
  state reducer instead of a loose `showConnection` flag.
- Compose screens render state and dispatch actions. They do not directly call
  repositories, token vault mutation, or connection clients.

## In Scope

- `apps/android/app/src/main/java/dev/nako/android/ui/NakoAndroidApp.kt`
- `apps/android/app/src/main/java/dev/nako/android/ui/connection/`
- `apps/android/app/src/main/java/dev/nako/android/ui/screens/settings/`
- focused JVM tests for state reducers/sessions
- workstream docs and closeout evidence

## Out Of Scope

- Changing public server connection contracts.
- Adding user accounts, OAuth/OIDC, or RBAC.
- Redesigning settings visuals.
- Replacing Compose or adding a dependency injection framework.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing `ServerProfileRepository` is sufficient as the domain mutation helper. | High | Connection and settings already use it for upsert, switch, and failure recording. | If insufficient, add small repository methods rather than duplicating mutation logic. |
| Connection testing can stay behind `NakoConnectionClient`. | High | It already returns safe success/failure results and has tests. | If future login/session auth appears, open a new auth/session workstream. |
| JVM tests can cover the architectural behavior without emulator smoke. | Medium | Browse and Player UDF lanes were covered by JVM gates for state architecture. | If visual behavior changes materially, run existing smoke harness separately. |

## Architecture Direction

Use the same pattern that worked for Browse and Player:

- pure session/reducer modules own state transitions and async orchestration;
- production adapters wrap `ServerProfileStore`, `TokenVault`, and
  `NakoConnectionClient`;
- Compose only observes session state and dispatches actions;
- root app state is explicit enough to test connection visibility after save,
  switch, sign-out, and reconnect actions.

Do not add AndroidX ViewModel or Hilt in this lane. The deep module is the
session; Android lifecycle adapters can remain thin.

## Closeout Condition

This lane can close when:

- connection setup no longer owns async test/save/switch orchestration inside a
  Composable;
- server profile settings no longer mutates repository/token vault directly in
  visual rendering;
- root app connection visibility is explicit and tested;
- focused and final Android JVM gates pass;
- docs record any remaining auth/session follow-ons.

## Closeout Notes

- `ConnectionSession` now owns connection form state, async connection testing,
  save, switch, failure recording, and token/profile persistence through a
  runtime adapter.
- `SettingsSession` now owns server profile switching and active-profile sign
  out. Settings visuals receive callbacks instead of mutating token vault or
  repository state directly.
- `NakoAppSession` now owns root snapshot and connection visibility. Connection
  setup, save, switch, and sign-out transitions all pass through explicit app
  actions.
- Browse and Player architecture were not changed by this lane.
- Auth/session/RBAC/token refresh remain future workstreams.
