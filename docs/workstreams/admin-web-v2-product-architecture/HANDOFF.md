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

## Active Task

- Task ID: AWV2-040
- Owner: unassigned
- Files: `apps/admin-web/src/components`, `apps/admin-web/src/features`,
  `docs/workstreams/admin-web-v2-product-architecture`
- Validation: `cd apps/admin-web && npm run check && npm run test && npm run build`
- Status: READY
- Review: extract only components proven by `/jobs`; do not start a broad
  design-system rewrite.
- Evidence needed: component tests or focused route tests plus browser smoke
  across desktop and mobile.

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
- Defer Tauri packaging until browser-hosted V2 route/data architecture is
  proven.

## Blockers

- None for AWV2-040.

## Next Recommended Action

Run AWV2-040:

- extract the shell, route header, table, filter bar, status badges, skeleton,
  and safe error patterns that `/jobs` proved;
- keep Admin API ownership inside `adminApi` and route/feature modules;
- keep `/legacy` route covered by a narrow availability test until its
  workflows migrate;
- decide whether any `shadcn-admin` pattern was copied directly. If yes, add
  provenance/license notes; if no, keep the current original shadcn-style
  composition note.
