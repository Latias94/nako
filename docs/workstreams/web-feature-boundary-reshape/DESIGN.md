# Web Feature Boundary Reshape

Status: Active
Last updated: 2026-05-28

## Why This Lane Exists

The v0 copy left dozens of large product components under `web/components/nako`.
That was acceptable for copy-first import, but it makes ownership unclear:
Media, Admin, setup, notifications, deferred domains, and shared UI concerns
live beside each other.

## Target State

- Product code is grouped by feature ownership under `web/src/features`.
- Shared UI remains DTO-free.
- API DTOs stay under `web/src/api` and are mapped before reaching shared UI.
- Large feature surfaces have smaller local modules and explicit public entry
  points.
- Deferred domains are isolated so they do not pollute live product surfaces.

## In Scope

- Move Media, Admin, setup/account, notification, and TV surfaces into feature
  directories.
- Keep shadcn/Radix primitives stable during the move.
- Add feature-level index files only when they clarify ownership.
- Update imports and tests.

## Out Of Scope

- Wiring new live APIs.
- Changing visual design.
- Route-owned child routes beyond the minimal import changes.

## Closeout Condition

This lane can close when feature boundaries are explicit, tests/check/build
pass, and no shared UI imports API DTOs.
