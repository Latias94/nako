# Source Duplicate Reconciliation Operator Flow

## Goal

Expose the shipped Admin source duplicate reconciliation plan/apply capability
through Admin Web so an operator can inspect redacted duplicate candidates and
explicitly create a Suggested Source Duplicate Relationship when the backend
recommends `suggest_relationship`.

## What I Already Know

- The backend and Admin API are already implemented:
  - `GET /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan`
  - `POST /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply`
- Generated Admin contract types and route keys already exist in
  `apps/admin-web/src/adminApi/generated/contract.ts`.
- Admin Web has no non-generated usage of these routes yet.
- Existing Admin Web patterns require:
  - live calls through `AdminApiClient` and `AdminDataSource`;
  - no direct page-level `fetch`;
  - controlled native forms and explicit confirmation for mutations;
  - mock fallback for unavailable reads;
  - tests for data-source calls, redaction, route behavior, and i18n when text
    changes.

## Requirements

- Add an operator-visible Admin Web flow for Source Duplicate Relationship
  reconciliation from an existing source context.
- Load the reconciliation plan through `AdminDataSource` and
  `AdminApiClient`.
- Render only redaction-safe plan facts:
  - source id;
  - duplicate source id;
  - evidence kind;
  - confidence;
  - stale state;
  - existing relationship id/status;
  - recommended action.
- Provide an explicit confirm step before calling apply.
- Enable apply only for `recommended_action === "suggest_relationship"`.
- Call apply with `expected_action: "suggest_relationship"` and the selected
  duplicate source id.
- Show success/replay result using redaction-safe response facts.
- Preserve existing item/library/source inventory behavior.
- Add deterministic mock data for unavailable live reads.
- Add tests covering:
  - plan route/data-source call;
  - successful apply payload and result rendering;
  - non-suggest actions do not call apply;
  - unsafe values are not rendered;
  - Chinese text for newly visible route copy.

## Acceptance Criteria

- [x] Admin Web exposes Source Duplicate reconciliation from a source row or a
      source-specific route.
- [x] Plan loading uses `AdminDataSource`; pages do not call `fetch` directly.
- [x] Mutation requires an explicit confirm action.
- [x] Apply is disabled for preserve/refresh recommendations.
- [x] Tests prove route/data-source calls and apply payload.
- [x] Tests prove raw locators, raw hashes, raw fingerprints, credentials, and
      durable input JSON are not rendered.
- [x] `npm run check --prefix apps/admin-web` passes.
- [x] Focused Admin Web tests pass.
- [x] `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-source-duplicate-reconciliation-operator-flow` passes.
- [x] `git diff --check` passes.

## Definition Of Done

- Implementation complete and tested.
- Evidence records commands and results.
- Spec update is completed or explicitly judged unnecessary.
- Changes are committed and pushed.
- Task is archived and session/goal status is closed.

## Technical Approach

Use the existing Item Detail source list as the entry point because it already
shows source rows, source ids, and related support links for playback
diagnostics. Add a source-scoped route such as
`/items/$itemId/sources/$sourceId/duplicates` or an equivalent Admin route that
receives `library_id` from the selected source facts. Keep the feature local to
Admin Web:

- extend `AdminApiClient` with plan/apply methods using generated route keys;
- extend `AdminDataSource` with a narrow feature method pair;
- add mock plan/response data;
- add a focused page/component for the plan, candidate list, confirmation, and
  result;
- add route tests in `App.test.tsx`.

## Decision

Prefer a source-scoped operator route reached from Item Detail rather than
embedding mutation controls directly inside every source row.

Consequences:

- Keeps Item Detail dense and scannable.
- Gives the repair flow enough space for explicit confirmation and errors.
- Keeps future duplicate relationship status or retry affordances localized to
  one feature page.

## Out Of Scope

- No backend route, DTO, schema, or generated contract changes unless a current
  contract bug is discovered.
- No automatic duplicate reconciliation.
- No Media Item merge.
- No relationship confirm/reject/undo flow.
- No broad Admin Web data-source refactor beyond what this feature needs.

## Technical Notes

- Specs read:
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
- Audit source:
  - `.trellis/tasks/archive/2026-06/06-06-06-06-current-function-architecture-audit/prd.md`

## Verification Evidence

- `npm run check --prefix apps/admin-web` passed.
- `npm run test --prefix apps/admin-web -- src/App.test.tsx src/adminApi/client.test.ts src/adminApi/dataSource.test.ts` passed: 3 files, 150 tests.
- `npm run test --prefix apps/admin-web` passed: 6 files, 174 tests.
- `npm run build --prefix apps/admin-web` passed; Vite reported only the existing large chunk warning.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-source-duplicate-reconciliation-operator-flow` passed.
- `git diff --check` passed.
- Dev server smoke check: `Invoke-WebRequest http://127.0.0.1:5173/items/item-unknown-1/sources/source-unknown-1/duplicates?library_id=library-anime` returned HTTP 200.
- Browser plugin smoke check was not available because the required Node REPL browser control tool was not exposed in this session.
