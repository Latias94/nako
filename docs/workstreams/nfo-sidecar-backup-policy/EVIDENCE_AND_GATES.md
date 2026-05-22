# NFO Sidecar Backup Policy Evidence And Gates

Status: Completed
Last updated: 2026-05-17

## Baseline Evidence

- M47 preserves unknown NFO XML fields during forced export.
- M48 makes NFO export use explicit atomic replace writes.
- M48 does not retain the previous NFO sidecar content before overwrite.
- WebDAV remains read-only for sidecar export in this slice.

## Focused Gates

```powershell
cargo fmt --all -- --check
cargo check -p nako-vfs --tests
cargo nextest run -p nako-vfs --no-fail-fast
cargo check -p nako-nfo --tests
cargo nextest run -p nako-nfo --no-fail-fast
```

## Closeout Gates

```powershell
cargo check --workspace --tests
cargo nextest run --workspace --no-fail-fast
git diff --check
```

## Evidence Log

- 2026-05-17: Workstream opened for M49.
- 2026-05-17: `cargo fmt --all -- --check` passed.
- 2026-05-17: `cargo check -p nako-vfs --tests` passed.
- 2026-05-17: `cargo nextest run -p nako-vfs --no-fail-fast` passed with 25
  tests.
- 2026-05-17: `cargo check -p nako-nfo --tests` passed.
- 2026-05-17: `cargo nextest run -p nako-nfo --no-fail-fast` passed with 18
  tests.
- 2026-05-17: `cargo check --workspace --tests` passed.
- 2026-05-17: `cargo nextest run --workspace --no-fail-fast` passed with 310
  tests.
- 2026-05-17: `git diff --check` passed.
- 2026-05-17: VFS write requests can ask for `StorageBackupMode::ExistingFile`,
  `LocalFsBackend` creates same-directory backups before overwrites, and NFO
  forced export reports backup creation/failure without changing public API
  routes.

## Expected Closeout Evidence

- VFS tests prove local backups are created before overwriting existing
  sidecars.
- VFS tests prove unsupported backup requests fail explicitly.
- NFO export tests prove forced overwrite requests and reports backup.
- NFO export tests prove fresh sidecar creation does not create backup.
- NFO export tests prove backup failure prevents final replacement.
- M47/M48 preservation and atomic write behavior still pass.
