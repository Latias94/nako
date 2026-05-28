# Web Test Harness And Route Contracts

Status: Complete
Last updated: 2026-05-28

## Why This Lane Exists

The frontend runtime is now Vite/TanStack/Tauri static, but `npm test` still
aliases `npm run check`. That is not enough protection for the next refactors:
moving feature boundaries, turning internal view-state into routes, deleting
fixture-only pages, and wiring live APIs.

## Target State

- `npm --prefix web run test` runs real Vitest tests.
- Route contract tests prove the shipped top-level paths render meaningful
  content: `/`, `/media`, `/admin`, `/notifications`, `/settings`, `/setup`,
  `/account`, and `/tv`.
- Public Client and Admin data-source tests prove fixture fallback and live
  client mapping boundaries.
- Test setup owns browser-only mocks such as `ResizeObserver`, localStorage,
  matchMedia, and layout APIs.
- Existing Playwright/static smoke remains the visible UI gate; Vitest becomes
  the fast regression gate.

## In Scope

- Add Vitest, Testing Library, jsdom, and focused tests.
- Replace the `test` script alias.
- Add test setup utilities only where they reduce repeated test noise.
- Keep route tests at product seams, not fragile DOM snapshots.

## Out Of Scope

- Full E2E coverage for every copied v0 page.
- Reorganizing feature directories.
- Wiring new live APIs.
- Visual regression testing.

## Architecture Direction

Tests should exercise public seams:

- route rendering through `AppRoot` or router exports;
- Public Client media data source through its exported factory;
- Admin dashboard data source through its exported factory;
- browser storage configuration through stable connection keys.

Avoid testing internal component state that will be deleted in later lanes.

## Closeout Condition

This lane can close when real Vitest tests pass, `npm test` is no longer a
type-check alias, route/data-source coverage exists, and the broader Vite/Tauri
gates still pass.
