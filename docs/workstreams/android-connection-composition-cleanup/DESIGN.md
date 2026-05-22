# Android Connection Composition Cleanup

Status: Closed
Last updated: 2026-05-20

## Problem

The root app composition lane introduced `NakoAppEnvironment` as the owner of
Android platform dependencies. `NakoConnectionShell` still has an unused
platform-building Composable that constructs profile store, token vault,
transport, and connection client directly.

`NakoConnectionShellContent` also accepts store/vault/client separately and
builds `ClientConnectionRuntime` internally. That keeps connection composition
shallower than browse/root composition and leaves a second path for dependency
graph drift.

## Target State

- Delete the unused platform-building `NakoConnectionShell` entrypoint.
- Let `NakoAppEnvironment` create `ConnectionRuntime`.
- Let `NakoConnectionShellContent` accept `ConnectionRuntime` and an explicit
  initial snapshot.
- Keep connection UI rendering and `ConnectionSession` behavior unchanged.
- Keep preview behavior through a tiny preview runtime.

## Scope

- `apps/android/app/src/main/java/dev/nako/android/ui/NakoAppComposition.kt`
- `apps/android/app/src/main/java/dev/nako/android/ui/NakoAndroidApp.kt`
- `apps/android/app/src/main/java/dev/nako/android/ui/connection/NakoConnectionShell.kt`
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

The lane closed with `NakoAppEnvironment.createConnectionRuntime()` as the
single production path for connection runtime composition. The unused
platform-building `NakoConnectionShell` entrypoint was removed.

`NakoConnectionShellContent` is internal and now accepts `ConnectionRuntime`
plus an explicit `ServerProfileSnapshot`. It no longer knows about platform
store/vault/client assembly. The preview still builds a local preview runtime.
