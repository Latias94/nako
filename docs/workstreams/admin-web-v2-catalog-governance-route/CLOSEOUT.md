# Admin Web V2 Catalog Governance Route Closeout

Status: Closed
Closed: 2026-05-25

## Closeout Claim

This lane is complete. Admin Web V2 now has a real `/catalog/governance` route
that replaces the placeholder with a route-first, read-only governance queue
backed by generated Admin API query DTOs and deterministic fallback behavior.

This closeout does not claim Catalog Governance workflow parity. Item detail,
repair actions, provider-mapping decisions, split/merge flows, and mutation
semantics remain follow-ons.

## Delivered

- New `apps/admin-web/src/features/catalog/CatalogGovernancePage.tsx`.
- `/catalog/governance` route wiring with URL search validation.
- `AdminApiClient.getCatalogGovernanceItems(query?: AdminCatalogGovernanceItemsQuery)`.
- `AdminDataSource.loadCatalogGovernance(query)` with section-local fallback.
- Query preservation for numeric zero values in Admin API query serialization.
- V2 filter bar for `library_id`, `max_confidence_milli`, and `limit`.
- V2 table for Media Item, kind, Media Library, Local Inference, issues,
  source count, and provider mapping acceptance.
- Tests for query mapping, fallback, route rendering, and unsafe text output.
- Desktop and mobile browser smoke screenshots under
  `target/admin-web-v2-catalog-governance-smoke/`.

## Review Result

### Workstream Compliance

- Blocking: none.
- Important: none.
- The route stays read-only and does not add detail or repair semantics.
- `/legacy` remains available.
- Follow-ons are explicit instead of hidden in this lane.

### Code Quality

- Blocking: none.
- Important: none.
- Admin API access remains behind `AdminDataSource`.
- Generated contract types are reused rather than duplicated.
- No broad design-system expansion was introduced.

### Missing Gates

- None for this lane's target state.

## Follow-ons

1. Catalog Governance detail route after item-level Admin API detail exists.
2. Repair/accept/split/merge mutation policy and UX.
3. Provider Mapping decision workflow once backend semantics are explicit.
4. Live-backend browser smoke for `/catalog/governance` once a local Admin API
   server is running during frontend verification.

## Evidence Anchors

- `docs/workstreams/admin-web-v2-catalog-governance-route/EVIDENCE_AND_GATES.md`
- `apps/admin-web/src/features/catalog/CatalogGovernancePage.tsx`
- `apps/admin-web/src/adminApi/client.ts`
- `apps/admin-web/src/adminApi/dataSource.ts`
- `apps/admin-web/src/App.tsx`
- `apps/admin-web/src/App.test.tsx`
- `apps/admin-web/src/adminApi/client.test.ts`
- `apps/admin-web/src/adminApi/dataSource.test.ts`
