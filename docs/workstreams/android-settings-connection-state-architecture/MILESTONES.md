# Android Settings Connection State Architecture - Milestones

Status: Closed
Last updated: 2026-05-20

## M0 - Lane Opened

Exit criteria:

- state architecture target is written;
- task ledger and validation gates are present;
- scope excludes auth/session product changes.

## M1 - Connection Session

Exit criteria:

- connection form/test/save/switch behavior lives behind a testable session;
- token persistence and profile snapshot persistence are runtime adapter
  responsibilities;
- connection UI renders state and dispatches actions.

## M2 - Settings Session

Exit criteria:

- server profile switch and sign-out behavior live behind settings session
  actions;
- settings visual surfaces no longer mutate token vault or repository directly;
- diagnostics remain token-safe.

## M3 - Root App State

Exit criteria:

- root app connection visibility is explicit and tested;
- save, reconnect, and sign-out transitions are handled through app/session
  state rather than ad hoc flags;
- Browse and Player integration remains stable.

## M4 - Closeout

Exit criteria:

- final Android JVM tests pass;
- `git diff --check` passes;
- workstream docs record shipped architecture and follow-ons;
- workstream status is closed.
