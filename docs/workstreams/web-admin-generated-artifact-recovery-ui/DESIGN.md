# Web Admin Generated Artifact Recovery UI

Status: Active
Last updated: 2026-06-02

## Why This Lane Exists

`generated-artifact-apply-operations-repair` shipped the backend and
data-source recovery queue, but operators still need a Web Admin page that can
be opened, filtered, inspected, and smoke-tested. Without that page the feature
is API-visible but not product-visible.

## Relevant Authority

- ADRs:
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
  - `docs/adr/0053-application-control-plane-boundary.md`
- Existing docs:
  - `docs/architecture/LANES.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
- Related workstreams:
  - `docs/workstreams/generated-artifact-apply-operations-repair/`
  - `docs/workstreams/web-admin-generated-artifact-review-mutations/`
  - `docs/workstreams/generated-artifact-provider-mapping-breadth/`

## Problem

The Web Admin Generated Artifacts area can review, apply, bulk apply, and show
batch results, but it does not yet have a dedicated recovery screen for
failed/stale/skipped/noop apply state. Operators must infer repair work from
batch detail or direct API calls.

## Target State

When this lane closes:

1. Web Admin has a route for Generated Artifact apply recovery.
2. The route loads the existing recovery read model and supports attention
   filtering.
3. The table distinguishes `needs_repair`, `needs_review`, `replay_only`, and
   `resolved` without presenting a blind retry action.
4. Route and data-source tests cover fixture and live response mapping.
5. Browser smoke proves the route renders on desktop and mobile without
   overflow or sensitive raw data.

## In Scope

- Add a Web Admin route for recovery queue browsing.
- Add route state normalization for attention, limit, and offset.
- Add an operator-dense table with status badges, summary counters, and links
  back to apply/outcome context where existing routes support it.
- Add focused Web tests and browser smoke.
- Update workstream and architecture docs.

## Out Of Scope

- No repair mutation or retry button.
- No backend API/schema changes unless the UI proves the current read model is
  insufficient.
- No Public Client API changes.
- No provider-depth precision or Provider Mapping conflict workflow.
- No generic job retry UI.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The GAOR recovery read model has enough fields for a useful first UI. | High | `docs/workstreams/generated-artifact-apply-operations-repair/CLOSEOUT.md`; `web/src/api/admin/read-models-data-source.ts` | If false, split a backend read-model follow-up instead of adding mutation. |
| A read-only route should precede repair mutation. | High | GAOR closeout and ADR 0053 control-plane boundary | If false, mutation scope must open a separate workstream with stronger idempotency evidence. |
| The route belongs to `web-product` with library metadata coordination. | High | `docs/architecture/LANES.md`; `docs/architecture/WORKSTREAM_LINKS.md` | If false, planner should reroute before implementation. |

## Architecture Direction

Use the existing Web Admin data-source pattern:

- route components consume read models, not raw Admin API DTOs;
- route state owns filters and pagination;
- fallback fixture mode remains explicit;
- live mode uses generated Admin contracts through `AdminApiClient`;
- page UI is read-only and leaves repair mutation to a separate follow-on.

## Closeout Condition

This lane can close when:

- the recovery route is implemented and linked from Admin Generated Artifacts;
- route/data-source tests and type checks pass;
- browser smoke covers desktop and mobile rendering;
- no sensitive raw internals are visible;
- repair mutation remains split to `proposed:generated-artifact-apply-repair-actions`.
