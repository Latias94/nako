# M2 Source Duplicate Suggestion Read-Only Review

## Goal

Make the existing Source Duplicate Relationship review flow easier for an
operator to discover and evaluate from Admin Web item detail pages, without
adding automatic reconciliation, schema changes, or new backend queue/list
surfaces.

## What I Already Know

- `CONTEXT.md` defines hard-linked or byte-identical files in separate
  libraries as separate Media Sources connected by a Source Duplicate
  Relationship, not as one merged source or one automatically merged item.
- The backend and Admin API already ship source-scoped reconciliation plan and
  guarded apply routes:
  - `GET /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan`
  - `POST /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply`
- Admin Web already has:
  - route `/items/$itemId/sources/$sourceId/duplicates`;
  - `SourceDuplicateReconciliationPage`;
  - feature-owned `sourceDuplicateReconciliationData` adapter;
  - an Item Detail source-row link to open duplicate review;
  - route tests for URL search, confirmation, mock read-only behavior,
    non-suggestion read-only behavior, zh-Hans copy, and unsafe-field redaction.
- Architecture docs mark the M1 backend plan/apply and Admin Web operator flow
  as shipped. They explicitly warn not to reopen automatic duplicate
  reconciliation or Media Item merge without new blocker evidence.

## Requirements

- Improve the operator-facing discovery path from Item Detail so Source
  Duplicate review is visible as a review workflow, not just a small per-source
  link.
- Keep the existing source-scoped route and Admin API methods. This task should
  not add a global duplicate queue, a library-level list route, or generated
  contract changes.
- Add a compact review summary on the source duplicate reconciliation page that
  helps an operator understand the current page before deciding whether any
  candidate is actionable.
- The summary must derive only from the existing redaction-safe plan fields:
  candidate ids, evidence kind, confidence, stale state, existing relationship
  status, recommended action, and page metadata.
- Preserve current guarded mutation behavior:
  - apply remains available only for live Admin API results;
  - apply remains limited to `recommended_action === "suggest_relationship"`;
  - apply still requires an explicit prepare/confirm step;
  - non-suggestion rows remain read-only.
- Keep localized copy in the existing i18n catalogs, with zh-Hans coverage for
  new visible text.
- Keep Admin Web route/page code behind `AdminDataSource` and the
  feature-owned adapter. Do not call `fetch` or generated routes directly from
  page components.
- Preserve unsafe-field redaction. UI must not render raw Source Locators,
  local paths, backend URLs, credentials, raw hashes, raw fingerprints, durable
  input JSON, or raw job material.

## Acceptance Criteria

- [x] Item Detail contains a more prominent Source Duplicate review entry that
      links to the first available source with its `library_id`, while source
      rows still expose per-source review links.
- [x] The source duplicate reconciliation route shows a compact review summary
      with counts for actionable suggestions, preserved/read-only candidates,
      stale/refresh candidates, and total returned candidates.
- [x] Existing apply confirmation behavior still works and remains live-only.
- [x] Mock/fallback plans remain read-only and do not fabricate successful
      mutation behavior.
- [x] Route tests cover the improved Item Detail entry, review summary counts,
      zh-Hans copy for new text, and redaction of unsafe values.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] Focused Admin Web tests for changed route/page behavior pass.

## Definition Of Done

- Tests are added or updated for the new operator-visible behavior.
- Admin Web typecheck/lint is green for the affected package.
- Trellis context files are configured for implement/check agents.
- Any spec update is either completed or explicitly judged unnecessary during
  finish.
- Work is committed with a Conventional Commit message when implementation and
  checks are complete.

## Technical Approach

Keep this as an Admin Web productization slice. The current backend contract is
already deep enough for this step; the missing leverage is operator locality:
the Item Detail page should tell operators that duplicate review exists, and
the review page should summarize what kind of action is possible before the
operator scans every candidate row.

Implementation shape:

- Extend Item Detail support links with a Source Duplicate review entry for the
  first source. This makes the workflow discoverable alongside playback support,
  artwork, generated artifacts, and catalog governance.
- Keep each source row's exact source-scoped duplicate review link.
- Add review-summary helper logic inside the source duplicate feature page or a
  small feature-local helper if the JSX would become noisy.
- Reuse existing `DataPanel`, `Badge`, `RouteNotice`, `EmptyRouteState`,
  `SourceLabel`, and native route/search patterns.
- Update `sourceDuplicate` and `itemDetail` i18n catalogs only for new copy.
- Prefer focused route tests in `App.test.tsx` and existing feature adapter
  tests if the adapter behavior changes.

## Decision (ADR-lite)

**Context**: A global queue/list could be useful later, but the repository
currently has a source-scoped plan/apply route and already shipped source-row
entry points. Adding a global queue now would require new backend listing
contracts and broader product semantics.

**Decision**: Implement the M2 increment as Item Detail discoverability plus a
source-scoped review summary, reusing existing Admin API plan/apply behavior.

**Consequences**: This improves the practical operator workflow with low schema
and API risk. It does not solve bulk review or fleet-wide duplicate triage; a
future queue can be opened when there is a concrete need for library-level or
global duplicate governance.

## Out Of Scope

- No automatic Source Duplicate Relationship reconciliation.
- No Media Source or Media Item merge.
- No relationship confirm/reject/undo workflow.
- No global duplicate review queue or library-level duplicate list route.
- No backend route, DTO, schema, migration, generated contract, or source-hash
  scheduling changes.
- No Public Client API behavior.

## Technical Notes

- Relevant existing files:
  - `apps/admin-web/src/features/items/ItemDetailPage.tsx`
  - `apps/admin-web/src/features/items/SourceDuplicateReconciliationPage.tsx`
  - `apps/admin-web/src/features/items/SourceDuplicateReconciliationRoutePage.tsx`
  - `apps/admin-web/src/features/items/sourceDuplicateReconciliationData.ts`
  - `apps/admin-web/src/i18n/catalogs/sourceDuplicate.ts`
  - `apps/admin-web/src/i18n/catalogs/itemDetail.ts`
  - `apps/admin-web/src/App.test.tsx`
- Relevant shipped predecessor:
  - `.trellis/tasks/archive/2026-06/06-06-source-duplicate-reconciliation-operator-flow/prd.md`
- Relevant constraints:
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
  - `docs/architecture/M1_ADMIN_DIAGNOSTICS_REPAIR_COVERAGE.md`
  - `docs/architecture/LANES.md`

## Verification Evidence

- `npm run check --prefix apps/admin-web` passed.
- `npm run test --prefix apps/admin-web -- src/App.test.tsx` passed: 1 file,
  135 tests.
- `npm run test --prefix apps/admin-web` passed: 8 files, 242 tests.
- `npm run build --prefix apps/admin-web` passed.
- `git diff --check` passed with Windows line-ending warnings only.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-14-06-14-m2-source-duplicate-suggestion-review`
  passed.
- Dev server smoke check:
  `http://127.0.0.1:5173/items/item-unknown-1` returned HTTP 200.

## Spec Update Review

No durable `.trellis/spec/` update is needed for this slice. The work follows
the existing Admin Web route/search/i18n/test and feature-owned adapter
contracts already documented in
`.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`; it does not add a
new API signature, schema, cross-layer contract, or reusable frontend pattern.
