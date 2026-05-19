# Android Player Effect Scope Cleanup - Milestones

Status: Closed
Last updated: 2026-05-19

## M0 - Lane Setup

Exit: APESC-010 complete.

## M1 - Scope Injection

Exit:

- Player exit work is launched from an app-owned effect scope.
- `PlaybackPlayerRoute` no longer constructs its own detached exit scope.
- Existing exit semantics remain unchanged by focused tests.

Status: Complete.

## M2 - Closeout

Exit:

- Focused JVM tests and diff checks pass.
- Workstream docs record evidence and residual follow-ons.

Status: Complete.
