# Admin Web live-only mutation guard

## Goal

Prevent Admin Web mutation workflows from acting on deterministic mock or hybrid
fallback read data. If the page's governing read model or plan did not come
from a fully live Admin API response, prepare/confirm controls must be disabled
or the mutation must reject before calling the data source.

## Requirements

- Add a shared Admin Web live-source guard helper for `AdminSectionResult`
  style values and `DataSourceMode`.
- Apply the guard to high-risk mutation workflows where a read plan can fallback
  to mock while the mutation method still exists:
  - Generated Artifact review.
  - Catalog Governance Provider Mapping review.
  - Item Artwork select/unpublish.
  - Source Duplicate Reconciliation apply.
- Keep existing mutation availability checks for missing data-source methods.
- Disable prepare/confirm controls when the route source is not live.
- Keep mutation functions guarded as a final safety net.
- Add focused route/data tests proving mock fallback does not call mutation
  methods.

## Acceptance Criteria

- [ ] Pages with mock/hybrid plan data cannot invoke their mutation data-source
      methods.
- [ ] Live pages still allow the existing prepare/confirm mutation flows.
- [ ] Existing Addons/Jobs/Settings/Storage live-guard patterns remain intact.
- [ ] `npm run test --prefix apps/admin-web -- App.test.tsx adminApi/dataSource.test.ts features/items/sourceDuplicateReconciliationData.test.ts`
      passes, or narrower failing coverage is justified in evidence.
- [ ] `npm run check --prefix apps/admin-web` passes.
- [ ] `git diff --check` passes.

## Definition of Done

- Focused tests and type-check pass.
- Trellis context validates.
- Specs updated if a new reusable guard convention is established.
- Changes are committed and the task is archived.

## Technical Approach

Add a helper near `AdminDataSource`/`DataSourceMode` so feature pages can ask
whether a current section result is fully live. Use that helper both in render
logic and mutation functions. For page components, derive a `canMutate` boolean
from the relevant source mode plus method availability and feed it into
prepare/confirm buttons.

## Decision (ADR-lite)

**Context**: Admin Web intentionally uses deterministic fallback reads so
operators can inspect route shapes when the Admin API is unavailable. That same
fallback must not become authority for writes.

**Decision**: The page's governing read source is the write gate. `source:
"live"` permits mutation if the method exists; `source: "mock"` and `source:
"hybrid"` block mutation.

**Consequences**: Mutation workflows remain explicit and live-only without
removing read fallback. Future workflow pages should reuse the same guard.

## Out of Scope

- No backend route or DTO changes.
- No new mutations or UI workflows.
- No redesign of deterministic mock read data.
- No realtime/Admin activity stream work in this slice.

## Technical Notes

- Positive reference: `apps/admin-web/src/features/addons/AddonsPage.tsx`
  already checks `result.source !== "live"` before retrying Addon Task Runs.
- Other existing guards: Jobs, Settings, Storage.
- Candidate files:
  - `apps/admin-web/src/adminApi/dataSource.ts`
  - `apps/admin-web/src/features/automation/GeneratedArtifactReviewPage.tsx`
  - `apps/admin-web/src/features/catalog/CatalogGovernanceRepairPage.tsx`
  - `apps/admin-web/src/features/items/ItemArtworkGalleryPage.tsx`
  - `apps/admin-web/src/features/items/sourceDuplicateReconciliationData.ts`
  - `apps/admin-web/src/features/items/SourceDuplicateReconciliationPage.tsx`
  - `apps/admin-web/src/App.test.tsx`
