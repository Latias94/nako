# Android Connection Composition Cleanup

Status: Closed
Last updated: 2026-05-20

## Problem

The root app composition lane introduced `TaruAppEnvironment` as the owner of
Android platform dependencies. `TaruConnectionShell` still has an unused
platform-building Composable that constructs profile store, token vault,
transport, and connection client directly.

`TaruConnectionShellContent` also accepts store/vault/client separately and
builds `ClientConnectionRuntime` internally. That keeps connection composition
shallower than browse/root composition and leaves a second path for dependency
graph drift.

## Target State

- Delete the unused platform-building `TaruConnectionShell` entrypoint.
- Let `TaruAppEnvironment` create `ConnectionRuntime`.
- Let `TaruConnectionShellContent` accept `ConnectionRuntime` and an explicit
  initial snapshot.
- Keep connection UI rendering and `ConnectionSession` behavior unchanged.
- Keep preview behavior through a tiny preview runtime.

## Scope

- `apps/android/app/src/main/java/dev/taru/android/ui/TaruAppComposition.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/TaruAndroidApp.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/connection/TaruConnectionShell.kt`
- Focused connection/root tests if needed.
- Workstream docs under this directory.

## Non-Goals

- Do not change profile repository semantics.
- Do not change connection test behavior.
- Do not add auth/session refresh.
- Do not introduce a dependency injection framework.

## Architecture Direction

Composition should happen once at the root environment seam. Connection UI
should render a session backed by a `ConnectionRuntime`, not know how runtime
dependencies are assembled.

## Closeout Notes

The lane closed with `TaruAppEnvironment.createConnectionRuntime()` as the
single production path for connection runtime composition. The unused
platform-building `TaruConnectionShell` entrypoint was removed.

`TaruConnectionShellContent` is internal and now accepts `ConnectionRuntime`
plus an explicit `ServerProfileSnapshot`. It no longer knows about platform
store/vault/client assembly. The preview still builds a local preview runtime.
