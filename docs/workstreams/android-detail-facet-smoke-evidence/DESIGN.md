# Android Detail Facet Smoke Evidence

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

The Android detail screen renders metadata chips and relationship rows, but the
current smoke path stops before proving that these elements open server-backed
facet result routes. Nako needs evidence that detail metadata is navigable
media graph UI, not static decoration.

## Relevant Authority

- `CONTEXT.md`: Media Item, Canonical Metadata, People, Genres, Tags, and
  Public Client API terminology.
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/workstreams/android-smoke-regression-harness/`
- `docs/workstreams/android-local-resume-smoke-evidence/`

## Problem

`profile-with-media` currently proves Home, detail, source picker, local
resume, and player launch. It does not prove that detail-page Genre, Tag, or
Person entries use their stable ids to open `/genres/{id}/items`,
`/tags/{id}/items`, or `/people/{id}/items`.

## Target State

When this lane closes:

- The server-backed `Night Harbor` smoke fixture exposes at least one
  API-backed Genre, Tag, and Person relationship on detail.
- `profile-with-media` smoke captures detail metadata relationship evidence.
- The smoke path taps representative Genre, Tag, and Person targets and proves
  each facet route returns `Night Harbor` via Public Client API.
- Unsupported relationship families remain explicit API-gap behavior and are not
  claimed as implemented.

## In Scope

- Existing Android demo fixture data under `Start-DemoFixtureServer.ps1` if the
  current metadata is insufficient.
- `Smoke-Emulator.ps1` navigation and surface criteria for detail facet routes.
- Android docs and workstream evidence.

## Out Of Scope

- Public Client API shape changes.
- Server schema or ingestion behavior changes.
- Collection, Studio, Year, Item Kind, Library, or Series navigation.
- Golden screenshot diffing.
- CI/device-farm execution.

## Architecture Direction

Prefer the existing fixture metadata first: `Night Harbor` already declares
Genre `Mystery`, Tag `Lighthouse`, and a Person credit `Mira Vale`. The smoke
harness should use the UI like a user:

1. Open detail from Home.
2. Scroll to `Metadata`, capture Genre and Tag chips.
3. Tap `Mystery`, wait for the Genre facet route, capture `Related Media Items`
   and `Night Harbor`.
4. Return to detail and repeat for `Lighthouse`.
5. Scroll to `Cast & Crew`, tap `Actor / as Keeper` or the exposed credit row,
   and prove the Person facet route returns `Night Harbor`.

This keeps the slice aligned with the current Android UX and avoids synthetic
route calls that bypass the UI.

## Closeout Condition

This lane can close when:

- `profile-with-media` smoke captures the metadata relationship surfaces;
- Genre, Tag, and Person facet routes are proven from detail UI navigation;
- focused smoke and regression smoke pass;
- `git diff --check` passes;
- evidence and handoff docs name remaining relationship follow-ons.

## Closeout

Closed on 2026-05-19. The `profile-with-media` smoke state now proves
server-backed detail metadata navigation for Genre, Tag, and Person
relationships. It captures detail metadata, opens `Mystery`, `Lighthouse`, and
`Actor / as Keeper` from the UI, and verifies each facet result route returns
`Night Harbor` through Public Client API-backed results.

Final evidence:

- Focused smoke report:
  `apps/android/build/smoke/20260519-104315-profile-with-media-emulator-5554/report.md`
- Regression smoke report:
  `apps/android/build/smoke-regression/20260519-104729/report.md`
- Diff hygiene:
  `git diff --check`
