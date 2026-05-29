# Web Admin Acquisition Intake - Design

Status: Active
Last updated: 2026-05-28

## Problem

Nako already has backend Acquisition Intake diagnostics, Managed Import linkage,
and a closed Admin Web V2 route-first proof in `apps/admin-web`, but the new
`web/` shell does not expose this operator workflow. WBBP removed the fake Media
Downloads surface, so downloads can only reenter the frontend through the
Admin-owned intake boundary.

The risk is product drift: a Media downloads page would imply user download
management before Nako has a downloader provider protocol, mobile/offline
download policy, or public client contract. The correct first UI is an Admin
route that reads intake candidates and explains readiness without mutating a
library.

## Target State

When this lane closes:

- `/admin/acquisition/intake` is a real route in the new `web/` shell.
- The route owns normalized search params for `library_id`, `state`,
  `source_kind`, `managed_import_artifact_id`, `limit`, and `offset`.
- `web/src/api/admin` exposes an acquisition intake read model backed by
  `AdminAcquisitionIntakeCandidateListResponse`.
- Fixture mode remains useful for local development and tests; live mode calls
  the generated Admin API route when a connection is configured.
- The page renders target library, source kind, state, redacted source summary,
  size, diagnostics availability, Managed Import linkage, and timestamps.
- The page never renders raw locators, host paths, credentials, prompt bodies,
  or downloader internals.
- Route contracts, route-state tests, data-source contract tests, TypeScript
  check, bundle budget, and browser smoke evidence pass.

## Scope

In scope:

- `web/src/api/admin/*` acquisition intake data-source/read-model additions.
- `web/src/features/admin/*` Admin navigation, page, and display-only workflow.
- `web/src/shell/nako-router.tsx` route and search-param normalization.
- Tests under `web/src/test` for data-source contracts, route rendering, route
  state, and redaction-sensitive rendering.
- Workstream evidence and closeout.

Out of scope:

- Backend/Admin API route implementation.
- Downloader protocol clients, torrent/Usenet/RSS/browser integrations, or
  background watcher scheduling.
- Managed Import promotion/apply mutations.
- Public Client API or Media surface download UI.
- Reusing `apps/admin-web` source code directly; the old lane is reference
  behavior only, while `web/` owns its own components and tests.

## Architecture Direction

Keep the route narrow and read-only first:

- Generated Admin contracts stay the source of truth for DTO names and paths.
- `web/src/api/admin` maps contract DTOs into UI read models and owns live vs
  fixture fallback behavior.
- `web/src/shell/nako-router.tsx` owns route search validation and serialization.
- `web/src/features/admin` renders the route as an operator diagnostic page
  inside the Admin surface, not as Media client navigation.
- Mutations such as watch-folder discovery or candidate acceptance require a
  later task only after review-plan, permission, idempotency, and redaction
  rules are explicit.

## Prior Art

- `docs/workstreams/downloads-watch-folder-intake`
- `docs/workstreams/managed-import-staging`
- `docs/workstreams/admin-web-v2-acquisition-intake-route`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`

## Risk Plan

- Contract drift: start with `WAAI-020` and record the exact generated Admin
  DTOs/routes before implementation.
- Unsafe rendering: tests must assert that source locators and raw paths are not
  rendered.
- Fake product promise: keep the page read-only until mutation contracts are
  separately proven.
- Bundle growth: require `npm --prefix web run build:budget` for route slices.
