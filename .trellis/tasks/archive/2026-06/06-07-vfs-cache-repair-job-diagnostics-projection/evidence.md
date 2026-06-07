# Evidence: VFS Cache Repair Job Diagnostics Projection

Date: 2026-06-07

## Implementation

- Added `AdminJobDiagnostics` to `JobResponse` and `AdminJobListItem`.
- Projected diagnostics only for `JobKind::VfsCacheRepair`; non-VFS jobs omit
  the optional field.
- Parsed valid VFS repair `summary_json` into the existing safe
  `AdminVfsCacheRepairJobSummary` DTO.
- Added redacted failure diagnostics derived from error presence only:
  `failure_class: unknown`, `safe_message: "storage failure"`, retryability,
  and job status.
- Regenerated Admin TypeScript contracts:
  - `apps/admin-web/src/adminApi/generated/contract.ts`
  - `web/src/api/admin/generated/contract.ts`
- Updated `.trellis/spec/nako-api/backend/admin-and-public-contracts.md` with
  the Admin Jobs VFS repair diagnostics contract.

## Redaction Boundary

- No raw `input_json`, `summary_json`, or `error` is exposed.
- No raw storage URI, source locator, local path, backend URL, credential,
  token, etag, fingerprint, or URI digest is exposed.
- No routes, DB schema, scheduler/runtime behavior, retry policy, repair policy,
  cache invalidation, backend configuration, or library file writes were added.

## Verification

- `cargo check -p nako-api --tests` passed.
- `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts` passed.
- `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts` passed.
- `cargo fmt --all -- --check` passed after running `cargo fmt --all`.
- `cargo check -p nako-api -p nako-server --tests` passed.
- `cargo nextest run -p nako-api admin_contract --no-fail-fast` passed: 8 tests.
- `cargo nextest run -p nako-api job_response --no-fail-fast` passed: 2 tests.
- `cargo nextest run -p nako-api admin_job --no-fail-fast` passed: 2 tests.
- `cargo nextest run -p nako-server admin_v1_vfs_cache_repair_job_routes_enqueue_and_execute_without_payload_leaks --no-fail-fast` passed: 1 test.
- `cargo nextest run -p nako-server admin_v1_vfs_cache_repair_retry_requeues_failed_job_without_payload_leaks --no-fail-fast` passed: 1 test.
- `npm run check --prefix apps/admin-web` passed.
- `npm run check --prefix web` initially failed because `web/node_modules/hls.js`
  was missing while the dependency was present in `web/package-lock.json`.
  `npm install --prefix web` restored the missing package without Git changes;
  the rerun of `npm run check --prefix web` passed.
- `git diff --check` passed.
- `python ./.trellis/scripts/task.py validate .trellis/tasks/archive/2026-06/06-07-vfs-cache-repair-job-diagnostics-projection` passed after archive.
