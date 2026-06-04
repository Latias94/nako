# Evidence: VFS cache repair action preview first slice

## Implementation Summary

VFS cache repair diagnostics now expose a stable structured
`recommended_action` enum alongside the existing display-oriented
`operator_action` text. The action is derived in `nako-vfs`, mapped through the
Admin DTO, returned by `/admin/v1/storage/staging`, and regenerated into both
Admin TypeScript contracts.

The slice remains read-only. It does not add cache refresh, invalidation,
delete, retry queue, durable jobs, repository methods, or schema changes.

## Files Changed

* `crates/nako-vfs/src/lib.rs`
* `crates/nako-vfs/src/cache.rs`
* `crates/nako-api/src/admin/storage.rs`
* `crates/nako-api/src/admin_contract.rs`
* `crates/nako-server/src/http/admin.rs`
* `crates/nako-server/src/http/tests/system.rs`
* `apps/admin-web/src/adminApi/generated/contract.ts`
* `web/src/api/admin/generated/contract.ts`
* `.trellis/spec/nako-vfs/backend/quality-guidelines.md`
* `docs/architecture/STORAGE_VFS.md`

## Verification

* PASS: `cargo run -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
* PASS: `cargo run -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`
* PASS: `cargo fmt --all`
* PASS: `cargo fmt --all -- --check`
* PASS: `cargo check -p nako-vfs -p nako-api -p nako-server --tests`
* PASS: `cargo nextest run -p nako-vfs cache --no-fail-fast`
* PASS: `cargo nextest run -p nako-api admin_vfs_cache_summary_serializes_redacted_repair_preview --no-fail-fast`
* PASS: `cargo nextest run -p nako-api admin_contract --no-fail-fast`
* PASS: `cargo nextest run -p nako-server admin_v1_storage_staging_lists_filters_and_redacts_paths --no-fail-fast`
* PASS: `git diff --check`
* PASS: `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-06-04-vfs-cache-repair-action-preview-first-slice`

## Notes

* `recommended_action` values are `none`, `refresh_cache`,
  `fix_backend_configuration`, and `inspect_failure`.
* `operator_action` remains available as display prose for existing clients.
* Executable URI-scoped cache repair/remediation remains a follow-on.
