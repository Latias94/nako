# Research: VFS cache repair action preview

- Query: identify the smallest bounded implementation for the proposed VFS
  cache repair operator-actions lane.
- Scope: internal
- Date: 2026-06-04

## Findings

The existing repair diagnostic path is already read-only:

* `nako_vfs::VfsCacheRepairDiagnostic` derives repair classification,
  retryability, safe message, and operator action text from cache status or the
  latest stored VFS cache failure.
* `StorageService::latest_vfs_cache_repair_diagnostic` reads
  `VfsCacheRepository::get_latest_vfs_cache_failure`.
* `/admin/v1/storage/staging` maps the VFS diagnostic into
  `AdminVfsCacheRepairDiagnostic`.
* Existing API and server tests assert that the preview redacts raw paths,
  source locators, etags, fingerprints, tokens, and backend error details.

## Decision

Implement a structured action preview, not an executable repair action.

The stable enum should be owned by VFS because it is derived from VFS repair
classification, then mapped into the Admin DTO. This avoids UI clients parsing
`operator_action` prose and keeps future executable remediation separate.

## Write Scope

* `crates/nako-vfs/src/lib.rs`
* `crates/nako-vfs/src/cache.rs`
* `crates/nako-api/src/admin/storage.rs`
* `crates/nako-server/src/http/admin.rs`
* `crates/nako-server/src/http/tests/system.rs`
* `crates/nako-api/src/admin_contract.rs`
* `apps/admin-web/src/adminApi/generated/contract.ts`
* `web/src/api/admin/generated/contract.ts`
* `.trellis/spec/nako-vfs/backend/quality-guidelines.md`
* `docs/architecture/STORAGE_VFS.md`

## Guardrails

* Do not add cache mutation routes or repository methods.
* Do not expose cache URI or raw storage errors.
* Do not change storage schema or Admin route inventory.
* Regenerate Admin TypeScript contract output from `nako-api` if the Admin DTO
  shape changes; do not hand-edit generated output.
