# Source duplicate suggestion and review surface

## Goal

Move Source Duplicate Relationship handling from discovery-only review into a
minimal auditable operator workflow. Administrators should be able to create a
Suggested relationship from fresh fingerprint evidence, then explicitly confirm
or reject that existing Suggested relationship without merging Media Sources,
changing Playback Source Selection, or exposing source locators, raw
fingerprints, paths, tokens, or job payloads.

## What I Already Know

- Domain language distinguishes Source Duplicate Relationship from merged Media
  Source identity. Duplicate sources do not automatically collapse into one
  Media Item.
- Existing backend and Admin Web already support a source-scoped duplicate
  reconciliation plan plus guarded `suggest_relationship` apply.
- Existing Admin Web route is
  `/items/$itemId/sources/$sourceId/duplicates?library_id=...`.
- Existing backend route is
  `POST /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply`.
- Existing spec requires the HTTP handler to stay thin and delegate duplicate
  freshness/action policy to `SourceDuplicateReconciliationAppService`.
- Current repository interface can upsert a canonical pair relationship, so the
  MVP can update Suggested to Confirmed/Rejected without adding schema.

## Requirements

- Extend source duplicate reconciliation apply with two additional explicit
  actions:
  - `confirm_suggested`
  - `reject_suggested`
- Allow these actions only when the current pair already has
  `SourceDuplicateRelationshipStatus::Suggested`.
- Preserve pair evidence fields and canonical pair identity while changing only
  relationship status to Confirmed or Rejected.
- Replay of an already Confirmed or Rejected pair should return a safe conflict
  instead of toggling or overwriting the relationship.
- Keep stale/missing/mismatched/cross-library validation redaction-safe.
- Admin Web should expose Confirm and Reject controls only for rows whose
  existing status is Suggested and whose route load source is live.
- Admin Web should keep mock/hybrid mutation unavailable behavior and must not
  fabricate successful confirm/reject mutations.
- Admin Web result copy should identify the applied status/action without
  rendering raw source material.

## Acceptance Criteria

- [x] App-service tests prove confirm and reject update only existing Suggested
      relationships and do not create duplicate rows.
- [x] App-service tests prove Confirmed/Rejected replay or toggling is rejected
      safely without writes.
- [x] HTTP route tests prove generated Admin request/response actions serialize
      and preserve Admin guard/redaction behavior.
- [x] Admin API contract generation includes the new expected actions.
- [x] Admin Web client/data-source/adapter route tests prove action payloads,
      live-only controls, i18n copy, result rendering, and redaction.
- [x] Existing `suggest_relationship` apply behavior remains green.

## Definition Of Done

- Rust formatting and focused server/API tests pass.
- Admin Web typecheck/tests/build pass for affected route/client/data-source
  code.
- Generated Admin Web contract is regenerated from `nako-api`, not hand-edited.
- Trellis task context and closeout evidence are updated.

## Technical Approach

Use the existing apply route and feature adapter rather than adding a new route:
the route already owns pair-scoped mutation, Admin guard, generated contract
coverage, and redaction tests. Extend the action enum and app-service match to
apply status transitions for existing Suggested relationships only.

Admin Web should generalize `applySuggestion` to a relationship action method
that accepts the candidate row action. The UI can keep a two-step prepare/confirm
interaction for all mutations.

## Decision (ADR-lite)

**Context**: Operators can now discover duplicate candidates and create
Suggested relationships, but there is no explicit review completion action.

**Decision**: Implement Confirm/Reject on existing Suggested relationships via
the existing duplicate reconciliation apply command.

**Consequences**: This keeps the MVP small and auditable, but does not add undo,
reopen, automatic merges, source variant grouping, or Media Item hierarchy
repair. Those require separate policy and likely additional audit/history
contracts.

## Out Of Scope

- Automatic Media Source or Media Item merging.
- Playback Source Selection changes.
- Undo/reopen from Confirmed or Rejected.
- Bulk confirm/reject.
- New schema or audit event table.
- Public Client API exposure.

## Technical Notes

- Specs:
  - `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
  - `.trellis/spec/nako-server/backend/http-api-patterns.md`
  - `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
- Existing implementation:
  - `crates/nako-server/src/app/source_duplicate.rs`
  - `crates/nako-core/src/media/source.rs`
  - `crates/nako-api/src/admin/operations.rs`
  - `crates/nako-server/src/http/admin.rs`
  - `apps/admin-web/src/features/items/SourceDuplicateReconciliationPage.tsx`
  - `apps/admin-web/src/features/items/sourceDuplicateReconciliationData.ts`

## Closeout Evidence

- `cargo check -p nako-api -p nako-server --tests`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server source_duplicate --no-fail-fast`
- `cargo nextest run -p nako-server admin_v1_source_duplicate_reconciliation --no-fail-fast`
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
- `cargo fmt --all`
- `npm run check --prefix apps/admin-web`
- `npm run test --prefix apps/admin-web -- src/features/items/sourceDuplicateReconciliationData.test.ts src/adminApi/client.test.ts src/adminApi/dataSource.test.ts src/App.test.tsx`
- `npm run test --prefix apps/admin-web`
- `npm run build --prefix apps/admin-web`
- `git diff --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-14-06-14-source-duplicate-suggestion-review`

## Spec Updates

- Updated `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
  so duplicate reconciliation apply now documents `suggest_relationship`,
  `confirm_suggested`, and `reject_suggested` as the supported action state
  machine.
