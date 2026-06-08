# Evidence

## Result

- Added shared Admin Web live-source helpers in `apps/admin-web/src/adminApi/dataSource.ts`.
- Guarded Generated Artifact review, Catalog Governance repair, Item Artwork select/unpublish, and Source Duplicate apply flows so mock or hybrid read data cannot authorize a live mutation.
- Kept render-level controls disabled for non-live sources and added mutation-function guards as the final safety net.
- Added regression tests for mock and hybrid fallback mutation blocking.
- Updated Admin Web frontend spec to require the shared live-source helpers for future mutation pages.

## Verification

- `npm run test --prefix apps/admin-web -- App.test.tsx adminApi/dataSource.test.ts features/items/sourceDuplicateReconciliationData.test.ts`
  - Passed: 3 files, 167 tests.
- `npm run check --prefix apps/admin-web`
  - Passed.
- `git diff --check`
  - Passed.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-08-admin-web-live-only-mutation-guard`
  - Passed.

## Notes

- The first focused test run exposed stale `:addon_id` path construction in `apps/admin-web/src/adminApi/dataSource.test.ts`; the generated route contract now uses `{addon_id}` templates, so the test helpers were aligned with the generated Admin API contract.
- A Trellis implement sub-agent was spawned but timed out after partial changes; it was closed and the main session completed the remaining implementation and verification.
