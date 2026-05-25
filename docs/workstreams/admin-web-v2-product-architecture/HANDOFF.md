# Admin Web V2 Product Architecture Handoff

Status: Active
Last updated: 2026-05-25

## Current State

AWV2-010 is complete. Root `PRODUCT.md` and `DESIGN.md` now give Nako a
project-level product/design context for product UI work. This workstream
inherits the completed Admin Web V0 baseline and the completed app-local Admin
API TypeScript contract lane.

AWV2-020 is complete. `ROUTE_API_READINESS.md` records route readiness, API
coverage, the `/jobs` first proof choice, route-owned search params, and the
required client/query changes.

AWV2-030 is complete. `apps/admin-web` now has a TanStack Router shell,
TanStack Query route-local server state, TanStack Table Jobs route proof,
shadcn-style UI primitives, URL-owned Jobs filters, deterministic section
fallback, and a legacy console route for workflows not yet migrated.

AWV2-040 is complete. The `/jobs` proof now consumes extracted layout, route,
filter, data panel, table, badge, skeleton, empty, and safe notice components.
V2 semantic tokens live under `apps/admin-web/src/design/tokens.css`.

## Active Task

- Task ID: AWV2-050
- Owner: planner
- Files: `docs/workstreams/admin-web-v2-product-architecture`
- Validation: final gates recorded freshly in `EVIDENCE_AND_GATES.md`
- Status: READY
- Review: decide whether to close this architecture lane or split focused
  implementation lanes by workflow.
- Evidence needed: final review notes and next-lane recommendations.

## Decisions Since Last Update

- Do not reopen `admin-web-console`; keep it as the completed V0 baseline.
- Do not reopen `admin-api-typescript-contract`; keep it as the completed
  generated contract lane.
- Keep Vite + React + TypeScript as the base stack.
- Use shadcn/ui-style dashboard composition as the early V2 UI baseline so
  feature delivery takes priority over custom product polish.
- Use the official shadcn dashboard example and `shadcn-admin` as references
  or selective extraction sources, not as unreviewed wholesale product clones.
- Prefer route-first V2 architecture before deeper UI work.
- `/jobs` is accepted as the first route proof.
- `AdminApiClient.getJobs()` must accept `AdminJobsQuery` and send generated
  search params instead of always reading the unfiltered route.
- Default browser `fetch` must be wrapped before storage on the API client so
  native fetch is not called as an unbound object method.
- Successful non-JSON Admin API responses should fail with a stable client
  error before JSON parsing so Vite/browser fallbacks do not expose parser
  internals in route notices.
- AWV2-040 copied no direct `shadcn-admin` source. Current shared components
  are original shadcn-style primitives and local compositions.
- Defer Tauri packaging until browser-hosted V2 route/data architecture is
  proven.

## Blockers

- None for AWV2-050.

## Next Recommended Action

Run AWV2-050:

- run review-workstream against AWV2-030 and AWV2-040 evidence;
- decide whether this lane should close or stay open for a second route;
- recommended follow-on lanes are Jobs, Media Libraries, Catalog Governance,
  Addons, or Settings, not a broad app rewrite;
- keep `/legacy` route until migrated workflows have their own route tests and
  browser evidence.
