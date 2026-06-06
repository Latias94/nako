# Admin Web Feature Data Adapter Deepening

## Goal

Deepen the Admin Web source duplicate reconciliation data boundary by moving
feature-specific plan/apply fallback and command mapping behind a small
feature-owned adapter. This keeps the route/page focused on UI state while the
broad `AdminDataSource` remains an integration boundary owned by App routing.

## What I Already Know

- The source duplicate operator flow shipped through
  `.trellis/tasks/archive/2026-06/06-06-source-duplicate-reconciliation-operator-flow/`.
- `SourceDuplicateReconciliationPage` currently accepts the entire
  `AdminDataSource` even though it only needs source duplicate plan/apply
  behavior.
- The current implementation keeps live calls behind `AdminDataSource` and
  uses mock fallback for unavailable plan reads.
- The page already has route, mutation confirmation, redaction, and i18n tests.
- The M1 release ladder fast gate passed, so this is an architecture locality
  slice rather than a product blocker fix.

## Requirements

- Add a feature-owned source duplicate reconciliation data adapter.
- The adapter must:
  - expose only plan loading, fallback plan creation, and explicit suggestion
    apply behavior;
  - delegate live behavior to the existing `AdminDataSource` methods;
  - preserve mock fallback behavior for unavailable plan reads;
  - preserve no-mock-success behavior for apply mutations.
- Update `SourceDuplicateReconciliationPage` so it depends on the small adapter
  interface, not the entire `AdminDataSource`.
- Update the route wiring in `App.tsx` to adapt `AdminDataSource` into the
  feature adapter.
- Add focused adapter tests proving delegation, fallback, and unavailable apply
  behavior.
- Preserve existing route behavior, i18n copy, confirmation flow, URL search
  handling, and redaction coverage.

## Acceptance Criteria

- [x] `SourceDuplicateReconciliationPage` no longer imports or accepts
      `AdminDataSource`.
- [x] A feature-owned adapter file exists under the source duplicate feature
      area and exposes a small typed interface.
- [x] Adapter tests prove live plan delegation, mock plan fallback, live apply
      delegation, and missing apply rejection.
- [x] Existing route tests for source duplicate reconciliation still pass.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] Focused Admin Web tests pass.
- [x] Trellis context validation and `git diff --check` pass.

## Definition Of Done

- Implementation is completed and tested.
- Task evidence records commands and results.
- Spec updates are completed if the adapter pattern should become durable
  guidance.
- Work is committed and pushed.
- Trellis task is archived.

## Technical Approach

Keep the broad `AdminDataSource` API stable for now. Add a feature adapter
factory that accepts a `Pick<AdminDataSource, ...>` and returns a small
`SourceDuplicateReconciliationDataAdapter` interface. Route wiring creates the
adapter and passes it to the page. The page keeps UI-owned translation and
confirmation state, while the adapter owns mock fallback and apply availability
checks.

## Decision

Start with source duplicate reconciliation only instead of a broad Admin Web
data-source refactor.

Consequences:

- The change improves locality where new M1 operator flows are likely to grow.
- Existing `AdminDataSource` route tests remain useful and do not need a wide
  rewrite.
- Future feature adapters can copy the pattern only when a workflow needs it.

## Out Of Scope

- No generated Admin API contract changes.
- No backend/API changes.
- No broad `AdminDataSource` split.
- No visual redesign.
- No route path, query parameter, or i18n copy changes.

## Technical Notes

- Specs:
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
- Skills:
  - `react-best-practices`
  - `react-hooks-best-practices`
- Primary files:
  - `apps/admin-web/src/features/items/SourceDuplicateReconciliationPage.tsx`
  - `apps/admin-web/src/App.tsx`
  - `apps/admin-web/src/adminApi/dataSource.ts`
  - `apps/admin-web/src/App.test.tsx`
  - `apps/admin-web/src/adminApi/dataSource.test.ts`

## Verification Evidence

- `npm run check --prefix apps/admin-web` passed.
- `npm run test --prefix apps/admin-web -- src/features/items/sourceDuplicateReconciliationData.test.ts src/App.test.tsx src/adminApi/dataSource.test.ts`
  passed: 3 files / 134 tests.
- `npm run test --prefix apps/admin-web` passed: 7 files / 178 tests.
- `npm run build --prefix apps/admin-web` passed. Vite reported the existing
  large chunk warning.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-admin-web-feature-data-adapter-deepening`
  passed with 5 implement context entries and 5 check context entries.
- `git diff --check -- .trellis/tasks/06-06-admin-web-feature-data-adapter-deepening .trellis/spec/admin-web/frontend/routes-forms-and-tests.md apps/admin-web/src/App.tsx apps/admin-web/src/features/items/SourceDuplicateReconciliationPage.tsx apps/admin-web/src/features/items/sourceDuplicateReconciliationData.ts apps/admin-web/src/features/items/sourceDuplicateReconciliationData.test.ts`
  passed. Git reported LF-to-CRLF working-copy warnings for existing markdown
  and TypeScript files, but no whitespace errors.
