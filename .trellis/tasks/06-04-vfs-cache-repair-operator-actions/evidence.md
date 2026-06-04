# Evidence: VFS cache repair operator actions

## Implementation Summary

The VFS cache repair Admin surface now exposes a latest action plan endpoint at
`GET /admin/v1/storage/vfs-cache/repair/action-plan`.

The plan classifies the latest unresolved repair diagnostic into:

* `no_action` when no repair diagnostic is active;
* `executable` when the existing `refresh_cache` route can be used;
* `plan_only` when the operator must fix backend configuration or inspect an
  unknown failure outside this API.

The existing `POST /admin/v1/storage/vfs-cache/repair/refresh-cache` mutation is
unchanged and remains the only executable repair action in this slice.

## Files Changed

* `crates/nako-api/src/admin/storage.rs`
* `crates/nako-api/src/admin_contract.rs`
* `crates/nako-server/src/app.rs`
* `crates/nako-server/src/app/storage.rs`
* `crates/nako-server/src/http/admin.rs`
* `crates/nako-server/src/http/tests/system.rs`
* `apps/admin-web/src/adminApi/generated/contract.ts`
* `web/src/api/admin/generated/contract.ts`
* `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`
* `docs/architecture/STORAGE_VFS.md`

## Verification

* PASS: `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
* PASS: `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`
* PASS: `cargo check -p nako-api -p nako-server --tests`
* PASS: `cargo fmt --all`
* PASS: `cargo nextest run -p nako-api admin_contract --no-fail-fast`
* PASS: `cargo nextest run -p nako-api admin_vfs_cache --no-fail-fast`
* PASS: `cargo nextest run -p nako-server vfs_cache_repair_action_plan --no-fail-fast`
* PASS: `cargo nextest run -p nako-server vfs_cache_refresh_action --no-fail-fast`
* PASS: `cargo fmt --all -- --check`
* PASS: `git diff --check`
* PASS: `python ./.trellis/scripts/task.py validate .trellis/tasks/06-04-vfs-cache-repair-operator-actions`

## Notes

* The action plan exposes Admin route key/path guidance only for
  `refresh_cache`; it does not expose target cache URI, source locator, local
  path, backend URL, etag, fingerprint, token, credential, or raw backend error.
* `fix_backend_configuration` and `inspect_failure` intentionally remain
  plan-only.
* URI-scoped previews and broader remediation planning remain follow-ons.
