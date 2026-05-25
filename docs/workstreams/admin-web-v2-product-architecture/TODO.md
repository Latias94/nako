# Admin Web V2 Product Architecture TODO

Status: Active
Last updated: 2026-05-25

## M0 - Context And Research Baseline

- [x] AWV2-010 [owner=codex] [deps=none] [scope=PRODUCT.md, DESIGN.md, docs/workstreams/admin-web-v2-product-architecture]
  Goal: Create root product/design context and open the Admin Web V2 research
  and product-architecture lane.
  Validation: docs agree on problem, target state, non-goals, authority, stack
  recommendation, and first proof candidate.
  Evidence: `PRODUCT.md`, `DESIGN.md`, this workstream.
  Handoff: Continue with AWV2-020 before changing `apps/admin-web`.

## M1 - Route-First IA And API Readiness

- [x] AWV2-020 [owner=codex] [deps=AWV2-010] [scope=docs/workstreams/admin-web-v2-product-architecture, apps/admin-web/src/adminApi]
  Goal: Freeze the V2 route map, route groups, first route proof, Admin API
  coverage, and route/query DTO gaps.
  Validation: route inventory documents live, mock, planned, and missing
  Admin API states for each V2 route.
  Review: review-workstream before implementation starts.
  Evidence: `ROUTE_API_READINESS.md`.
  Handoff: Continue with AWV2-030 using `/jobs` as the first route proof.

## M2 - First Route Architecture Proof

- [x] AWV2-030 [owner=codex] [deps=AWV2-020] [scope=apps/admin-web]
  Goal: Implement the first route-first proof, preferably `/jobs`, with a real
  router, route-local server state, URL filters, fallback state, shadcn-style
  table/filter composition, and focused tests.
  Validation: `cd apps/admin-web && npm run check && npm run test && npm run build`.
  Review: review-workstream for UI boundary, data boundary, and test coverage.
  Evidence: `apps/admin-web/src/App.tsx`,
  `apps/admin-web/src/features/jobs/JobsPage.tsx`, shadcn-style UI
  primitives under `apps/admin-web/src/components`, route/query tests,
  `target/admin-web-v2-smoke/{desktop,mobile}.png`.
  Handoff: Continue with AWV2-040. Keep `/legacy` available until the migrated
  route set covers the old console workflows, then delete it deliberately.

## M3 - Component And Design-System Extraction

- [ ] AWV2-040 [owner=unassigned] [deps=AWV2-030] [scope=apps/admin-web/src/components, apps/admin-web/src/design]
  Goal: Extract only the components proven by the first route proof: shell,
  route header, table, filter bar, status badges, skeletons, and safe error
  states, using shadcn/ui primitives as the default baseline.
  Validation: frontend check/test/build plus browser smoke across desktop and
  mobile viewports.
  Review: verify no nested cards, text overflow, inaccessible icon buttons, or
  unsafe sensitive data rendering.
  Evidence: component tests and Playwright/browser screenshots.
  Handoff: Decide whether any copied `shadcn-admin` patterns should remain
  local, be rewritten, or be replaced by direct shadcn/ui composition.

## M4 - Closeout Or Split

- [ ] AWV2-050 [owner=planner] [deps=AWV2-040] [scope=docs/workstreams/admin-web-v2-product-architecture]
  Goal: Close this design lane or split the next V2 implementation lanes by
  workflow.
  Validation: final gates are recorded freshly in `EVIDENCE_AND_GATES.md`.
  Review: review-workstream has no blocking findings.
  Evidence: `HANDOFF.md`, `WORKSTREAM.json`, final gate notes.
  Handoff: Recommended next lanes should be Jobs, Media Libraries, Catalog
  Governance, Addons, or Settings, not a broad app rewrite.
