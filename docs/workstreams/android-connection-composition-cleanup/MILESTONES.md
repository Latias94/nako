# Android Connection Composition Cleanup - Milestones

Status: Closed
Last updated: 2026-05-20

## M1 - Boundary Frozen

Status: Complete

Exit criteria:

- Cleanup target and non-goals are documented.

## M2 - Runtime At Root

Status: Complete

Exit criteria:

- `NakoAppEnvironment` creates `ConnectionRuntime`.
- Focused root composition tests pass.

## M3 - Duplicate Entry Removed

Status: Complete

Exit criteria:

- `NakoConnectionShellContent` accepts runtime directly.
- Unused platform-building `NakoConnectionShell` is gone.
- Focused connection/root tests pass.

## M4 - Closeout

Status: Complete

Exit criteria:

- Full Android debug unit tests pass.
- Diff hygiene passes.
- Workstream docs record final evidence.
