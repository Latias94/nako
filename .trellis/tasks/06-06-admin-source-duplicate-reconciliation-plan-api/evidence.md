# Admin Source Duplicate Reconciliation Plan API Evidence

## Implementation Summary

- Added redaction-safe Admin DTOs for source duplicate reconciliation plans:
  `AdminSourceDuplicateReconciliationPlanResponse` and candidate DTOs.
- Added generated Admin contract route key and TypeScript DTO types for
  `GET /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan`.
- Added the Admin HTTP route and handler in `nako-server`; the handler parses
  path ids and bounded pagination, delegates to
  `SourceDuplicateReconciliationAppService::plan_source_duplicate_reconciliation`,
  and maps through explicit `nako-api` DTO conversion.
- Regenerated both Admin TypeScript contract artifacts:
  `apps/admin-web/src/adminApi/generated/contract.ts` and
  `web/src/api/admin/generated/contract.ts`.
- Updated the server source fingerprint / duplicate reconciliation Trellis spec
  with the now-landed Admin read-only route contract.

## Validation

- `cargo check -p nako-api -p nako-server --tests` passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed:
  8 tests passed.
- `cargo nextest run -p nako-api source_duplicate --no-fail-fast` passed:
  1 test passed.
- `cargo nextest run -p nako-server admin_v1_source_duplicate_reconciliation_plan --no-fail-fast`
  passed: 4 tests passed.
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast`
  passed: 1 test passed.
- `cargo nextest run -p nako-server source_duplicate --no-fail-fast` passed:
  7 tests passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed with only Git line-ending warnings.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-admin-source-duplicate-reconciliation-plan-api`
  passed.

## Redaction And Boundary Checks

- Route must expose only plan IDs, evidence kind, confidence, stale state,
  existing relationship status/id, and recommended action.
- Route must not expose Source Locators, local paths, etags, backend URLs,
  credentials, raw hashes, raw fingerprints, evidence values, durable job input
  JSON, or source fingerprint material.
- Route must remain read-only and must not create or update
  `SourceDuplicateRelationship` rows.
- HTTP tests assert success, candidate-oriented pagination, safe errors for
  missing source / cross-library source / missing fingerprint / raw fingerprint,
  non-admin rejection, redaction, and unchanged relationship rows before/after
  planning.

## Workflow Notes

- Implemented as an independent follow-on after the 06-06 fearless refactor wave
  was archived.
- Trellis sub-agent spawning was not used because the available sub-agent tool
  requires explicit user authorization for delegation. Equivalent Trellis
  context loading, spec review, implementation, and focused verification were
  performed in the main session.
