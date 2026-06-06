# Evidence

## Summary

- Added Admin-only source duplicate reconciliation apply route:
  `POST /admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply`.
- Added request/response DTOs and generated Admin contract route key
  `sourceDuplicateReconciliationApply`.
- Implemented app-service-owned validation and mutation through
  `SourceDuplicateReconciliationAppService::apply_source_duplicate_reconciliation`.
- Documented the explicit apply boundary in
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`.

## Verification

- `npm run generate:admin-api --prefix apps/admin-web` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check -- .trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md apps/admin-web/src/adminApi/generated/contract.ts crates/nako-api/src/admin/operations.rs crates/nako-api/src/admin_contract.rs crates/nako-core/src/media/source.rs crates/nako-server/src/app.rs crates/nako-server/src/app/source_duplicate.rs crates/nako-server/src/app/tests/source_duplicate.rs crates/nako-server/src/http/admin.rs crates/nako-server/src/http/tests/system.rs web/src/api/admin/generated/contract.ts .trellis/tasks/06-06-admin-source-duplicate-reconciliation-apply-first-slice` passed.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-admin-source-duplicate-reconciliation-apply-first-slice` passed.
- `cargo check -p nako-core -p nako-api -p nako-server --tests` passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed: 8 passed.
- `cargo nextest run -p nako-server source_duplicate --no-fail-fast` passed: 12 passed.
- `cargo nextest run -p nako-server admin_v1_source_duplicate_reconciliation --no-fail-fast` passed: 7 passed.
- `cargo nextest run -p nako-server implemented_admin_routes_are_generated_or_explicitly_excluded --no-fail-fast` passed: 1 passed.
- `Compare-Object (Get-Content apps/admin-web/src/adminApi/generated/contract.ts) (Get-Content web/src/api/admin/generated/contract.ts) | Select-Object -First 20` produced no differences.

## Notes

- Apply only writes when the current pair recommendation is
  `suggest_relationship`.
- Replaying an already Suggested pair returns the existing relationship with
  `created: false`.
- Existing Confirmed and Rejected pairs are rejected as conflicts and preserved.
- Stale target or candidate evidence is rejected as a refresh recommendation
  before any relationship write.
- Responses and tested error bodies avoid source locators, paths, raw hashes,
  raw fingerprints, evidence values, etags, credentials, and durable job input.
