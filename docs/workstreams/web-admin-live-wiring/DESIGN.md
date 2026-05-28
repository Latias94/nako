# Web Admin Live Wiring

Status: Queued
Last updated: 2026-05-28

## Why This Lane Exists

The Admin dashboard has a live/fixture seam, but deeper copied pages remain
fixture/planned. Operators need real libraries, users, tasks/jobs, logs,
settings, and Addon Manager flows before the admin frontend is product-ready.

## Target State

- Admin feature pages use generated Admin API contracts through admin-only data
  modules.
- Each page has loading, empty, error, permission, and mutation states.
- Fixture fallback is explicit and not confused with live product status.
- Addon Manager UI uses Nako Addon vocabulary and API shape.
- Shared UI remains free of Admin DTO imports.

## In Scope

- Libraries, users, tasks/jobs, logs, settings, and Addon Manager first live
  slices.
- Focused data-source tests and route tests.
- Safety review for destructive Admin mutations.

## Out Of Scope

- Backend API design not already accepted by server/API workstreams.
- Downloads/acquisition and playback UI.
- Visual redesign.

## Closeout Condition

This lane can close when accepted Admin pages have live seams, tests/build pass,
and unsupported Admin-like copied controls are removed or marked planned.
