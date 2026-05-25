# Admin Web V2 Catalog Governance Route - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Exit criteria:

- Read-only route scope is explicit.
- Query params are chosen.
- Detail and repair mutations are out of scope.

Primary evidence:

- `DESIGN.md`
- `TODO.md`

## M1 - Route And Data Boundary

Exit criteria:

- `/catalog/governance` renders real route content.
- Route search params map into `AdminCatalogGovernanceItemsQuery`.
- Section-local fallback is deterministic.
- Unsafe text exclusions are tested.

Primary gates:

- `npm run check`
- `npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`

## M2 - Evidence And Closeout

Exit criteria:

- Full frontend gates pass.
- Browser smoke evidence is recorded.
- Detail/repair follow-ons are explicit.

Closeout result: complete on 2026-05-25.
