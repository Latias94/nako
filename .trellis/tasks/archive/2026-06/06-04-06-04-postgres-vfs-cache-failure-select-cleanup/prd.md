# PostgreSQL VFS cache failure SELECT cleanup

## Goal

Deduplicate the PostgreSQL VFS cache failure SELECT row shape so future
authority fields and SQL casts cannot drift between getter paths.

## Requirements

- Keep behavior unchanged.
- Keep the fix local to `nako-db` PostgreSQL VFS cache persistence.
- Preserve the existing `library_id::text AS library_id` cast required by the
  PostgreSQL contract.

## Acceptance Criteria

- Both VFS cache failure getter paths use one shared SELECT fragment.
- PostgreSQL storage-runtime contracts still pass.
- Focused SQLite VFS cache behavior remains green.

## Out of Scope

- No schema migration changes.
- No public API changes.
- No removal of legacy URI/root fallback behavior.

## Technical Notes

- This follows the existing local pattern of `VFS_CACHE_OBJECT_SELECT` and
  `STAGING_MANIFEST_RECORD_SELECT` in the same PostgreSQL adapter module.

## Verification

- Passed: `cargo check -p nako-db --tests`.
- Passed: `cargo nextest run -p nako-db vfs_cache --no-fail-fast`.
- Passed: `pwsh -File scripts/postgres-contract-harness.ps1 -Suite storage-runtime -DatabaseUrl postgres://...`.
- Passed: `cargo fmt --all -- --check`.
- Passed: `git diff --check`.
