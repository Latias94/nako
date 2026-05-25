# Admin Web V2 Product Architecture Milestones

Status: Active
Last updated: 2026-05-25

## M-AWV2.0 Context And Research Baseline

Objective:

- Establish root product/design context for future Admin Web work.
- Open a durable V2 product-architecture lane.
- Record current app shape, stack candidates, and first proof recommendation.

Deliverables:

- `PRODUCT.md`
- `DESIGN.md`
- `docs/workstreams/admin-web-v2-product-architecture/`

Exit criteria:

- Product register is explicit.
- V2 does not overwrite the completed V0 baseline.
- Stack recommendation distinguishes keep, add, selective add, and defer, and
  accepts shadcn/ui-style composition as the early feature-first UI baseline.
- The first implementation candidate is named.

Status: completed by AWV2-010.

## M-AWV2.1 Route-First IA And API Readiness

Objective:

- Turn route names into route ownership, URL state, data ownership, and API
  readiness.

Deliverables:

- Route inventory or updated `V2_RESEARCH.md`.
- First proof slice requirements.
- Explicit API gaps and follow-on split list.

Exit criteria:

- Every V2 route is classified as live, fallback-capable, mock, planned, or
  missing.
- Search params and pagination behavior are documented for the first proof.
- No implementation starts with ambiguous route/data ownership.

Status: completed by AWV2-020. `ROUTE_API_READINESS.md` selects `/jobs` as
the first proof and records route/API coverage.

## M-AWV2.2 First Route Architecture Proof

Objective:

- Prove a route-first Admin Web pattern on one narrow workflow.

Deliverables:

- First route proof in `apps/admin-web`.
- Route-local server state.
- shadcn/ui-style table, filter, button, badge, empty, and error composition.
- Focused tests.
- Browser verification evidence.

Exit criteria:

- Frontend check/test/build pass.
- Route works with live Admin API data and deterministic fallback data.
- Error, loading, empty, and redaction states are test-visible.

Status: completed by AWV2-030. `/jobs` now proves the route-first shell,
route-owned URL filters, route-local server state, generated
`AdminJobsQuery` usage, section fallback, shadcn-style table/filter
composition, and focused browser smoke evidence.

## M-AWV2.3 Component And Design-System Extraction

Objective:

- Extract only components that have real V2 pressure.

Deliverables:

- Shared route shell, header, table, filter, badge, skeleton, and safe error
  components.
- Documented token roles.
- Decision on whether `shadcn-admin` patterns should be copied selectively,
  rewritten, or avoided.

Exit criteria:

- Component vocabulary is consistent across the proof route.
- Responsive behavior is browser-checked.
- Component ownership does not leak Admin API concerns.

Status: completed by AWV2-040. `/jobs` now consumes extracted shell, route,
filter, data panel, table, badge, skeleton, empty, and safe notice components,
with V2 design tokens isolated in `apps/admin-web/src/design/tokens.css`.

## M-AWV2.4 Closeout Or Split

Objective:

- Close the research/design lane or split focused implementation lanes.

Deliverables:

- Updated `HANDOFF.md`.
- Updated `WORKSTREAM.json`.
- Final evidence notes.

Exit criteria:

- Remaining work is not hidden inside a broad V2 label.
- Next tasks are independently executable and reviewable.
