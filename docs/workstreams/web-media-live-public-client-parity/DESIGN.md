# Web Media Live Public Client Parity - Design

Status: Active
Last updated: 2026-05-28

## Problem

The new `web/` shell has route contracts, connection profile handling, and a
first Public Client live data source, but the Media surface still behaves like a
fixture-first imported v0 product in several important paths:

- home rails and library browse do not yet express live API limitations clearly;
- library-scoped browse and Recently Added need explicit route/API readiness;
- playback entry still uses a local player mock instead of a browser-ticket
  backed source;
- continue-watching and progress writes are not yet wired into the new shell;
- browser/Tauri verification is not part of the Media implementation loop.

## Target State

The Media surface should become a truthful video-first client:

- fixture mode remains available for local development and tests;
- live mode uses generated Public Client SDK methods where contracts exist;
- missing live contracts are shown as explicit readiness gaps, not fake content;
- playback starts only through accepted browser-ticket or session APIs;
- user playback state reads/writes go through Public Client routes;
- route contracts, data-source tests, browser smoke, Tauri build, and bundle
  budgets are required before closeout.

## Scope

In scope:

- `web/src/api/public/*` live data-source expansion;
- `web/src/features/media/*` route-owned live/fixture state surfaces;
- route tests and data-source contract tests under `web/src/test`;
- browser/Tauri validation gates;
- documentation of Public Client route/API gaps.

Out of scope:

- Admin routes, acquisition/downloads, AI, automation, playlists, music, photos,
  podcasts, or mobile-native UI;
- backend route implementation unless a tiny contract generation fix is required
  and isolated;
- restoring deleted v0 prototypes;
- changing bundle thresholds except with recorded evidence.

## Architecture Direction

The Media surface should stay a thin client over Public Client contracts:

- `web/src/api/public` owns DTO-to-UI mapping and fixture/live fallback policy.
- Route components own URL state and pass stable initial view state into
  feature surfaces.
- Feature components do not import Admin API clients or privileged server state.
- Playback uses browser-safe URLs/tickets and never exposes raw local paths,
  source locators, or bearer tokens in media element URLs.
- Tauri-specific playback remains a follow-on until the browser flow is proven.

## Risk Plan

- Backend contract drift: require SDK/OpenAPI evidence before claiming live
  support.
- Fake playback: keep the mock player labeled as readiness-only until browser
  tickets are wired.
- Bundle regression: keep `build:budget` in every implementation task.
- Dirty worktree risk: current playback/subtitle backend edits are unrelated to
  this docs opening task and must not be staged by this lane.

