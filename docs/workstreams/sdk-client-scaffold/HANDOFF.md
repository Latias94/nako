# SDK Generation And Client Integration Scaffold Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M33 is closed. `taru-api` now emits a dependency-free TypeScript/Web/CLI SDK
scaffold from the OpenAPI v1 contract, and tests cover public route inventory,
auth/version/error/pagination runtime behavior, and admin/internal leakage
rejection.

## Decisions Since Last Update

- First target is TypeScript/Web/CLI because it can be generated as a
  dependency-free `fetch` wrapper without adding Node or Dart tooling.
- Dart/Flutter SDK generation remains a follow-on.
- Generated code is emitted by command, not published as an npm package in
  this slice.
- No Node/Dart dependencies were added.

## Blockers

- None.

## Next Recommended Action

- Open M34 for TypeScript SDK package hardening and contract compile checks if
  the next goal continues the client integration path.
- Keep Dart/Flutter SDK generation, npm/pub publishing, OpenAPI runtime
  serving, and concrete client UI as separate follow-ons.
