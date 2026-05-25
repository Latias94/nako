# Admin Web V2 Route And API Readiness

Status: Draft
Last updated: 2026-05-25

This document completes AWV2-020. It records which V2 routes can be built from
current Admin API contract data, which routes need fallback or mock data, and
which route is the first refactor proof.

## Route Readiness Matrix

| V2 route | Data status | Current contract/API | V2 implementation note |
| --- | --- | --- | --- |
| `/overview` | Live plus fallback | `GET /admin/v1/overview` | Keep as a summary route after shell refactor. |
| `/jobs` | Live plus fallback | `GET /admin/v1/jobs`, `AdminJobsQuery`, `AdminJobListResponse` | First proof route. Client must accept query params. |
| `/jobs/:jobId` | Missing Admin route | Legacy known-ID job detail exists outside generated Admin route constants | Defer detail route until Admin API detail is generated. |
| `/automation/events` | Live plus fallback | `GET /admin/v1/events` | Needs query support before table route. |
| `/libraries` | Partial | public library routes plus `GET /admin/v1/libraries/{library_id}/metadata-profile` | V2 list can start read-only, but create/edit is not ready. |
| `/libraries/:libraryId` | Partial | public library detail/source routes | Needs route-owned source list and scan/NFO action policy. |
| `/libraries/:libraryId/metadata-profile` | Live plus mutation | generated `libraryMetadataProfile` route and DTOs | Good second or third route after jobs because update semantics are already designed. |
| `/catalog/governance` | Live plus fallback | `GET /admin/v1/catalog/governance/items` | Good table route after jobs, but repair remains out of scope. |
| `/catalog/governance/:itemId` | Missing | no generated detail route | Defer until item-level governance detail exists. |
| `/metadata/providers` | Partial | provider diagnostics exist outside current generated admin route list | Keep mock/hybrid until contract coverage is reconciled. |
| `/metadata/maintenance` | Partial | maintenance plan/job routes exist outside current generated admin route list | Defer route-first implementation until contract coverage is reconciled. |
| `/playback/sessions` | Live plus fallback | `GET /admin/v1/playback/sessions`, generated list DTO | Needs query support before table route. |
| `/playback/sessions/:sessionId` | Partial | `GET /admin/v1/playback/support` can support evidence by session/source query | Build after jobs if support-evidence UX is prioritized. |
| `/playback/runtime` | Live plus fallback | `GET /admin/v1/playback/runtime` | Can be a diagnostics page with cards and tables. |
| `/storage/staging` | Live plus fallback | `GET /admin/v1/storage/staging` | Needs query support before table route. |
| `/automation/generated-artifacts` | Live plus fallback | `GET /admin/v1/automation/generated-artifacts/proposals` | Review/accept routes exist in generated contract but require stronger UX policy. |
| `/addons` | Live plus mutations | `/admin/v1/addons` list/detail/status routes | Existing feature-rich area should be split after route shell exists. |
| `/addons/new` | Live mutation | `POST /admin/v1/addons` | Existing manifest paste flow can become a dedicated route. |
| `/addons/:addonId` | Live plus mutations | detail, health-check, surfaces, install-guide, diagnostics | Good later workflow, too broad for first proof. |
| `/addons/:addonId/tokens` | Live plus mutations | token list/issue/rotate/revoke routes | Requires one-time token UX and redaction tests. |
| `/addons/:addonId/grants` | Live plus mutation | grant list/replace routes | Requires permission and Library-Scoped Addon Grant UX. |
| `/network` | Live diagnostics | `GET /admin/v1/system/config` | Read-only diagnostics route. |
| `/settings` | Live diagnostics, mutation missing | `GET /admin/v1/system/config` | Keep read-only until settings mutation semantics exist. |

## First Proof Route

Use `/jobs` as the first route-first proof.

Why this route:

- generated contract already exposes `AdminJobsQuery` and `AdminJobListResponse`;
- rows are operational and easy to verify without new mutations;
- filters map naturally to URL search params;
- redaction requirements are simple and testable;
- the current client lacks query support, giving the proof a real but narrow
  data-boundary improvement.

## `/jobs` Search Params

Initial route-owned search params:

- `status`
- `kind`
- `resource_class`
- `library_id`
- `source_id`
- `limit`
- `offset`

Rules:

- omit empty string values from Admin API requests;
- parse `limit` and `offset` as positive integers with safe defaults;
- keep unknown search params out of the Admin API request;
- preserve filter state in the URL so reload and share links are meaningful;
- keep fallback mock data deterministic when the Admin API route fails.

## `/jobs` UI Assembly

Use shadcn/ui-style composition:

- app shell/sidebar from shadcn dashboard/admin patterns;
- route header with source/fallback state;
- filter bar using select/input/button primitives;
- data table with status badges and row affordances;
- loading skeleton, empty state, and safe error state;
- no bespoke Nako visual system beyond tokens, copy, icon, and semantic state.

`shadcn-admin` may inform shell, sidebar, data-table, and settings patterns.
Do not import it as an unreviewed product clone. If code is copied directly,
record MIT/provenance notes in the implementation slice.

## Required Code Changes For AWV2-030

- Add route and server-state dependencies.
- Change `AdminApiClient.getJobs(query?: AdminJobsQuery)` to use
  `withQuery(NAKO_ADMIN_ROUTES.jobs, query)`.
- Add a route-owned jobs query adapter and tests.
- Split shared shell/status/table primitives out of `App.tsx`.
- Keep old dashboard content available behind a fallback or legacy route until
  enough routes are migrated, then delete it.
- Update tests from single `App` assertions toward route-level tests.

## Follow-On API Gaps

- Generated Admin API job detail route.
- Query support in `AdminApiClient` for events, playback sessions, storage
  staging, and catalog governance.
- Catalog governance item detail and repair plan routes.
- Library list/detail Admin API route ownership and runtime edit policy.
- Settings mutation policy.
